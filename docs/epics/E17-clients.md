# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12); M1 done (2026-10-13: T1, T2, T3 and T6 on 2026-10-12, T5, T4, T7 and T8 on 2026-10-13); M2 done (2026-10-14: T1, T3, T4a, T6 and T4b-1 on 2026-10-13, T4b-2a, T2, T5a, T5b and T5c on 2026-10-14); M3 in progress (T1a–T1c, T2, T3a, T3b, T4a, T4b-1, T4b-2a, T4b-2b-1, T4b-2b-2, T5a, T5b-1, T5b-2a, T5b-2b, T6a, T6b, T6c-1, T6c-2, T7a-1, T7a-2a, T7a-2b, T8, T7b-1, T7b-2a, T7b-2b-1, T7b-2b-2a, T7b-2b-2b, T7b-2b-3, T7c-1, T7c-2, T7c-3, T7d, T9a, T9b-1, T9b, T9c done); the adventure lane's harness H1 and H2 done (2026-10-19). Refined 2026-10-12 (E17a, the asset notes,
done the same day). Decisions: [ADR-0020](../adr/0020-clients.md) (accepted at the M0 checkpoint,
point 7 rewritten there). Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`,
created by the first task that has detail to record; done tasks in full:
[`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset notes
are there). Next: T9d (CLT-Q9 to Q17's answers built, M3's last task), then M4 T0 (sieges
refined), M5 live play after it (`docs/lanes/client/QUEUE.md`). Since T9c (2026-10-23) this epic
is the battle client's; the adventure map client is E24, the map-client lane's.

## Goal

Clients that show the engine's games with the original's own assets, truthful to the original
down to the pixel and the frame: first a battle replay viewer, then sieges and playable battles.
Everything with graphics, sound, input or a window lives in `clients/`, outside the core. The
adventure map, the full game record/replay viewer and whole games played are E24's (the
map-client lane, since 2026-10-23), on the same foundation.

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
  through a committed catalogue of keys. **The packs are nested JSON** with i18next's conventions
  (2026-10-19, M3 T4b-1, over TOML and Fluent), and **a translation of the original's text is
  never committed**: a player may load a pack of their own, which the project neither ships nor
  hosts (CLT-Q7, 2026-10-19). An ADR-0020 amendment is drafted in `docs/lanes/client/adr-drafts/`.
- **Graphics options** (2026-10-13), eventually: per effect the original's way or a modern one
  (shadows first); `faithful` is the default and the reference. The draw list names what is
  drawn, the compositor decides how (QUEUE inbox).
- **The original's redraw artifacts are not copied** (2026-10-14, M3 T1a): the original redraws
  by dirty rectangles (design §7.11), which darkens shadows again and leaves stale count boxes;
  the viewer keeps drawing each frame whole in every profile, and the frame check classifies
  those pixels apart (a pixel that is ours with one to five more shadows) while any other
  difference still fails (M3 T1c, done). An ADR-0020 point 9 amendment is drafted in
  `docs/lanes/client/adr-drafts/` for the next checkpoint.
- **The original's drawing bugs are fixed, and reproduced only for the comparisons** (2026-10-15,
  M3 T4a): a bug in the original's drawing code (the first: the text drawer loses the glyph at a
  long word's break) is fixed in what the client draws; the comparisons with the original's
  captures (the capture set, the frame check) draw it as the original does, so that they stay
  exact. A development switch (`h3_render::Bugs`), neither a player's option nor a quirk
  profile. Goes into the ADR-0020 point 9 amendment draft. The second (2026-10-20, M3 T6c-1):
  the software cursor drawn at an even x (0x50CE04, design §9.9). The third to fifth
  (2026-10-22, M3 T7c-1, CLT-Q8: "dev-only for the captures, fixed otherwise"; fixed in T7d,
  the switch reaching the timeline: `anim::Stage::bugs`): a creature caster's group by the
  angle's wrong sign, a caster its spell missed kept on its casting frame until the whole
  redraw, the target's highlight kept through a creature's frames.
- **Every oddity of the original is an open question** (the user, 2026-10-22: so that nothing is
  forgotten): a drawing quirk or bug a reading or a recording finds becomes a `CLT-Q<n>` in
  `docs/OPEN-QUESTIONS.md` in the same task (copy it, fix it with the original's only for the
  comparisons, or a player's option; a recommendation), and the review list names it.
- **The client lane split** (2026-10-23, M3 T9c, CLT-Q18): with the rendering basics done, the
  user split the lane: this lane (`client`) keeps the battle client, M3 closing with T9d, then
  **M4 sieges, then M5 live play**; a new lane, `map-client` (ids `MCL`), builds the adventure map
  client in parallel as **E24**. The client foundation (`h3-assets`, `h3-render`, `h3-sdl`, the
  workspace's files and gates, the string catalogue) and the capture harness became common paths
  (`lanes.toml`): either client lane changes them in its own pull request, an additive change
  freely, a change in behaviour noted in the other's inbox; the adventure lane changes the
  harness without the `cross-lane` label.
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

- The scene over time completed (T1a, T1b-1, T1b-2; M2's T4b-2b): turns, the walk-by, deaths,
  defending, shots up and down, the **rays** (the user: Arch Magi and Beholders draw an energy
  ray, not a projectile), and recordings that check the provisional rules (the flier's layer, the
  shot's loop, the breath's groups); Haste and Slow with the spells (T7b; T1b re-cut with the
  user, 2026-10-15).
- A representative situation set over the harness: every battlefield (T2), the heroes (T3a) and
  war machines (T3b), the count boxes (T5) and the panel (T6), each mask removed as it is drawn;
  sieges later.
- Fonts and the text drawer (T4a, with the count boxes in the stills), the string catalogue and
  the language packs (T4b-1, an ADR-0020 amendment; T4b-2a the log's lines read and composed,
  T4b-2b a line drawn), and the battle interface (T5, T6: the boxes over time, the panel, the
  log, the cursors). T4 split with the user on 2026-10-15, T4b and T4b-2 on 2026-10-19.
- The **spell animations** (the user, at T5c: their plethora; T7a the code reading, split with
  the user on 2026-10-21 into T7a-1, the reading, and T7a-2, the harness's casts and the
  recordings; T7b on stacks with morale and luck, split with the user on 2026-10-21 into
  T7b-1, a hero's cast on one stack, and T7b-2, the rest, which the user split on 2026-10-21
  into T7b-2a, the parts without the original's rolls, and T7b-2b, morale, luck and the
  creatures' rolled animations, which the user split on 2026-10-21 into T7b-2b-1, the harness
  forcing the original's rolls (the user's wish), T7b-2b-2, morale, luck and fear (its first part
  T7b-2b-2a, a scenario's hero's luck and morale on the original's map, at the user's word), and
  T7b-2b-3, the creatures' rolled animations; T7c area spells, rays and spell obstacles, split
  with the user on 2026-10-22 into T7c-1, the area and mass spells, T7c-2, the spell obstacles and
  Armageddon, and T7c-3, raised stacks, the creatures' own animations and Chain Lightning) and
  their sounds.
- The sound's remainder (T8: the battle's end; the volume settings later).
- **Exit:** the reference set passes, or each difference is a listed fix or a classified quirk.
- **T1a done 2026-10-14** ([log](E17-log.md#m3-t1a--turns-and-the-walk-by-2026-10-14),
  [design §7.10, §7.11](E17-design.md)): a battle in progress in the original (hot seat, the
  harness plays both sides and acts as each turn starts); `climb` and `behind` recorded: a walk's
  later flip, a melee whose striker and target both turn, a shot whose shooter turns; both
  timelines as recorded on the first try, frames 43 of 43 and 164 of 194 exact. The walk-by
  difference read: the original redraws by dirty rectangles (shadows drawn again, the saved
  screen's count boxes); the user's decision: not copied, the frame check classifies them
  (**T1c**; Decisions).
- **T1c done 2026-10-15** ([log](E17-log.md#m3-t1c--the-frame-check-classifies-redraw-artifacts-2026-10-15),
  [design §7.11](E17-design.md)): a differing pixel is a redraw artifact when the original's is
  ours with one to four more shadows (or an earlier moment's with at least one more); the frame
  files record `<ms> <differing> <artifacts>`. The six sequences captured again, every count as
  recorded: `behind`'s 30 frames and 6 of `melee`'s 7 artifacts only; `melee` at 3 966 keeps the
  stale count box (231 pixels) listed for T5.
- **T1b-1 done 2026-10-15** ([log](E17-log.md#m3-t1b-1--shots-and-rays-2026-10-15),
  [design §7.12, §7.4](E17-design.md)): the ray read (`draw_ray` and its helpers) and built in
  integers (`h3_view::ray`); the hot-seat sequence `shots` (three rays, the Cyclops' throws, a
  defend reaction, a death by a shot) as recorded, all 139 frames equal to the original's outside
  the masks; a shooter now plays its group out beyond the reactions.
- **T1b-2 done 2026-10-15** ([log](E17-log.md#m3-t1b-2--flights-the-breath-the-mouse-and-the-set-by-time-2026-10-15),
  [design §7.13](E17-design.md)): the hot-seat sequence `fly` (two flights across rows, a
  breath on two stacks) as recorded on the first run, its frames exact or redraw artifacts only:
  a flier's layer 8 and the breath's groups and detection lifted from *provisional*; the mouse
  over a stack read (`battle_mouse_stack` 0x477C20: group 1 at once, the outline 0x70) and
  recorded (`hover`), the window's mouse-over queued for T6; the capture set compares each frame
  at its time, all 17 exact outside the masks.
- **T2 done 2026-10-15** ([log](E17-log.md#m3-t2--every-battlefield-2026-10-15),
  [design §5, §6.2](E17-design.md)): the special battlefields keyed by the packs (`base` 5, `sod`
  7) and drawn by the observation's `battlefield` over the terrain's picture; the shore and the
  nine special grounds set up in the original and captured, all exact (11 frames, the shore once
  with an absolute obstacle); the deck and the two boats drawn from scenarios, their capture
  waiting for boats in `h3-map`'s writer (CLT-Q5).
- **T3a done 2026-10-15** ([log](E17-log.md#m3-t3a--the-heroes-2026-10-15),
  [design §4.5, §7.14](E17-design.md)): T3 split with the user, the war machines left as T3b.
  The heroes' battle pictures read (by the class's town and the hero's sex, 18 DEFs; Conflux by
  sex, not by class as VCMI) and harvested, each side's flag in its player's colours and hero
  drawn, the flags and the heroes' idle motion, defeat and victory over time; the `heroes` mask
  gone from the capture set (28 situations, 44 frames, all exact: every hero picture, the idle
  motion) and from the frame check (the heroes from the original's memory).
- **T3b done 2026-10-15** ([log](E17-log.md#m3-t3b--the-war-machines-2026-10-15),
  [design §7.15](E17-design.md)): the war machines drawn as stacks (no count box, no fidget) and
  numbered as the original's records; an automatic machine's turn ticks the battle loop once;
  the First Aid Tent's heal under magic animation 79 (`PlayMagicAnimation` read for one stack,
  the translucent drawer 0x47EF60, the table 0x641E08 harvested). Captured: `war-machines` and
  the smoke battle (30 situations, 55 frames, all exact) and the hot-seat sequence `machines`
  (two Ballistas' shots, the heal: timeline and 106 frames exact).
- **T4a done 2026-10-15** ([log](E17-log.md#m3-t4a--fonts-the-text-drawer-and-the-count-boxes-2026-10-15),
  [design §3.6, §3.7, §4.3](E17-design.md)): T4 split with the user (the catalogue left as T4b).
  The fonts read and decoded (`h3_assets::fnt`, the gate decodes all nine), the executable's
  text drawer ported with its line breaking and quirks (`h3_render::text`), its tint in exact
  integers (`h3_render::tint`), each stack's count box drawn after its sprite (the layout's
  `[count_box]`). The capture set without the count-box mask: 31 situations, 57 frames, all
  exact (the new `counts`: three to five digits).

- **T4b-1 done 2026-10-19** ([log](E17-log.md#m3-t4b-1--the-string-catalogues-format-2026-10-19),
  [design §9](E17-design.md)): T4b split with the user (the log's text drawn left as T4b-2). The
  catalogue built (`h3_view::text`): the committed key catalogue `clients/text/keys.toml` (the
  battle log's first keys, the creatures' names as a family over the asset tables' original
  ids), packs as nested JSON (i18next's conventions, CLDR's integer plural rules), the original's
  pack read at run time from the user's tables, a player's pack before it (CLT-Q7); the gate reads
  all 155 keys' 306 forms; `h3-view text` formats the log's lines from them.
- **T4b-2a done 2026-10-19** ([log](E17-log.md#m3-t4b-2a--the-battle-logs-lines-read-and-composed-2026-10-19),
  [design §9.4](E17-design.md)): T4b-2 split with the user (the line drawn left as T4b-2b). The
  log's lines read in the executable (`battle_log_add`, the strike's line and its callers, wait,
  defend, morale, luck): its row n is record n, a name's form and a row's by the number of exactly
  1; the keys lifted from *provisional* (the war machine's row added, the bad luck row dropped:
  the original has none); `h3_view::battle_log` composes them and the entries a line becomes;
  the gate composes 12 sample lines from the user's tables, `h3-view text --log` prints them.
- **T4b-2b-1 done 2026-10-19** ([log](E17-log.md#m3-t4b-2b-1--the-logs-item-in-the-panel-2026-10-19),
  [design §9.5](E17-design.md)): T4b-2b split with the user (a strike's line against a capture
  left as T4b-2b-2). The panel's log item read (a text item over `cRollovr.pcx` at (214, 563),
  400 × 32, `smalfont.fnt`, colour 1, centred; two entries as its text) and drawn (the layout's
  `[log]`, `Draw::Log`); the capture set compares it (`panel_beside_log`), the harness's cursor
  parked beside it: 31 situations, all frames exact.
- **T4b-2b-2 done 2026-10-19** ([log](E17-log.md#m3-t4b-2b-2--a-strikes-line-against-a-capture-2026-10-19),
  [design §9.6](E17-design.md)): narrowed with the user to one line in a still (when it appears,
  several entries and the scroll left to T6). The hot-seat sequence `strike`: the Archangel's
  fixed damage kills all ten Pikemen, so the engine's line is the original's; the viewer draws
  the record's first strike's line (`battle_log::shown`, the small font's line count) from the
  sequence's `log_at`, the frame check compares the log's item from then on: exact.
- **T5a done 2026-10-20** ([log](E17-log.md#m3-t5a--the-count-boxes-through-walks-and-flights-2026-10-20),
  [design §4.3](E17-design.md)): T5 split with the user (the boxes whose numbers never change
  first; the numbers over time T5b; the spells' tint to T7b). +0x30 read (set through a walk and
  a flight) and built (`Pose::boxless`), a dead stack's hex freed at the strike's end; `walk`,
  `turn`, `climb` and `strike` compare their boxes (`boxes = true`): artifacts only, but 134
  pixels of the original's redraws at `strike`'s 2 730 ms, listed.
- **T5b-1 done 2026-10-20** ([log](E17-log.md#m3-t5b-1--the-count-boxs-number-over-time-2026-10-20),
  [design §4.3](E17-design.md)): T5b split with the user (`shot` and `melee` first; the other
  four sequences' boxes T5b-2). +0x50 read (the count before a damage's deaths, −1 again at the
  strike's end, whose whole redraw shows it at once) and built (`anim::CountKey`,
  `Moment::counts`; the strike's end shows its pending poses at once); the frame check masks and
  lists a box only where its count is not the original's; `shot` and `melee` (run to the strike
  back's end, 7 000 ms) compare their boxes: 40 to 37 and 30 to 12 exact, artifacts only.
- **T5b-2a done 2026-10-20** ([log](E17-log.md#m3-t5b-2a--the-count-boxes-in-behind-and-shots-2026-10-20),
  [design §4.3](E17-design.md)): T5b-2 split with the user (`behind` and `shots` first; `fly`
  and `machines` T5b-2b). Both compare their boxes; `behind` runs to the Marksmen's second
  shot's end (4 000 ms); a shot's struck stacks take layer 5 as in a melee (the original's
  memory); listed: the first arrow's steps over the rocks (the original's redraws, §7.11).
- **T5b-2b done 2026-10-20** ([log](E17-log.md#m3-t5b-2b--the-count-boxes-in-fly-and-machines-2026-10-20),
  [design §4.3, §7.15](E17-design.md)): both sequences at once (the user). Every sequence with
  frames now compares its boxes; a magic animation is drawn after its stack's count box
  (`draw_battle_stack`'s part 0 runs on from the sprite into the box: the heal's frames lie over
  the patient's box in `machines`, now exact in all 106 frames); the heal never changes a box's
  number (it lowers +0x58 only); `fly`'s breath kills differ from the original's (8 and 13
  against 9 and 16): both boxes listed from 4 300 ms.
- **T6a done 2026-10-20** ([log](E17-log.md#m3-t6a--the-panels-still-2026-10-20),
  [design §9.7](E17-design.md)): T6 split with the user (the panel and the log over time T6b; the
  status line, the cursors and the mouse T6c), who asked for reusable UI components. The
  executable's dialog items read (a picture, a DEF button and its states, a text over a picture)
  and built as components (`h3_view::ui`, the layout's `[panel]` as a list of items), the panel's
  base and `battle_controls_state` read in full and the controls' states at a turn's start built
  (`h3_view::panel`); the capture set compares the whole panel: 31 situations, 57 frames exact
  (the game's parked cursor masked); `drawn` digests what the comparisons see.
- **T6b done 2026-10-20** ([log](E17-log.md#m3-t6b--the-panel-and-the-log-over-a-battle-2026-10-20),
  [design §9.8](E17-design.md)): the user took it whole. The log's state read (new entries,
  the 3 000 ms status rule, settling, the scroll) and where each line is written; the timeline
  records what the battle writes and when, `battle_log::Log` keeps it, `scene::Panels` draws the
  panel through the battle's turns (kept colours and scroll buttons). The harness keeps panel
  changes and reads the log: the ten sequences compare the panel, every count as recorded, the
  log's item exact in 228 frames, listed in 668 (hints, T6c; rolled lines).
- **T6c-1 done 2026-10-20** ([log](E17-log.md#m3-t6c-1--the-mouses-still-2026-10-20),
  [design §9.9](E17-design.md)): T6c split with the user (the still T6c-1, over time T6c-2). The
  battle's mouse read (where it is, the cursor's kind and frame, the attack's twelve slots, the
  hints over the field and the panel, the software cursor's hot spots; its even x a drawing bug
  reproduced for the comparisons only, the user) and built as
  `h3_view::mouse` over the legal commands; stills draw the cursor, the hint and the marked
  stack's outline. 26 mouse situations exact from 100 ms; the set's parked cursor drawn, its
  masks gone: 57 situations, 85 frames exact.
- **T6c-2 done 2026-10-21** ([log](E17-log.md#m3-t6c-2--the-mouse-over-time-2026-10-21),
  [design §9.10](E17-design.md)): taken whole (the user). When the battle takes the mouse (a
  human's waiting turn; queued through actions, posted again at a human's turn), the hints in
  the log (forced or not, the hex-change rule, the target left over), entering a stack (its
  group 1 on the idle steps, stood by an action, its partial redraw), the software cursor's
  33 ms poll; built as `mouse::track` over each turn's battle from the engine, played through the
  sequences (their hint listings gone, the cursor drawn, `hover` compared) and the window (the
  mouse, pressed buttons, the hotkeys, the log's scroll); a ray hides the cursor. Every
  timeline as recorded; the sequences captured afresh with the cursor compared: 922 frames, the
  recorded counts unchanged, the hint listings gone (173 rolled lines left of 668), `hover` 16
  of 20 frames exact (four with the mouse's partial redraw's leftover pixels).
- **T7a-1 done 2026-10-21** ([log](E17-log.md#m3-t7a-1--the-magic-animations-read-2026-10-21),
  [design §7.16](E17-design.md)): T7a split with the user (the reading; the harness's casts and
  the recordings T7a-2). Which of seven players draws each spell's and creature's animation (one
  stack, the marked stacks, several, a raise, a hex, the field, their own drawers), each
  player's timing, the hero's cast frames around the spell, a creature caster's group, morale,
  fear and luck with their sounds and log lines; 16 map entries added. Candidates for T7a-2's
  recordings: animations a call names but never draws, a mass animation's missing last frame, a
  caster's group by an angle unlike the shot's.
- **T7a-2a done 2026-10-21** ([log](E17-log.md#m3-t7a-2a--the-harness-casts-2026-10-21),
  [design §7.16](E17-design.md)): T7a-2 split with the user (the harness first; the other
  recordings T7a-2b). The spell book read; the scenarios' heroes carry books; `cast:` in the
  harness and the viewer; two proofs recorded (`situations/casts.toml`): a hero's cast as read,
  a Faerie Dragon's group by its own angle confirmed (review list).
- **T7a-2b done 2026-10-21** ([log](E17-log.md#m3-t7a-2b--the-magic-animations-recorded-2026-10-21),
  [design §7.16](E17-design.md)): every player of §7.16 recorded (seven sequences in
  `casts.toml`, `dragon` again with its screens: the harness's `screens`, the box where the screen
  changed). The times read held; corrected by the recordings: Chain Lightning's 38 and Fire
  Shield's 11 drawn over marked stacks, the projectile's DEF by the opposite angle, a mass
  reaction's hit running on into standing. Confirmed: a mass animation's last frame never shown,
  Animate Dead's 50, the Liches' plain cloud; new: Chain Lightning's 37 never on the screen, a
  Fire Wall's second hex placed at the first's (review list).
- **T8 done 2026-10-21** ([log](E17-log.md#m3-t8--the-battles-end-tune-2026-10-21),
  [design §8.2](E17-design.md)): the battle's end read (the result's window picks one of six
  tunes by the side shown, the outcome and a town; played once, the effects ended, when the next
  turn would start) and recorded (`end`: decided at 3 730 ms, the viewer's timeline as recorded);
  the window plays the tune; the start sound and the heal at their loader's volume, 127. The
  volume settings and a track resuming wait for the client's options (later).
- **T7b-1 done 2026-10-21** ([log](E17-log.md#m3-t7b-1--a-heros-cast-on-one-stack-2026-10-21),
  [design §7.16](E17-design.md)): T7b split with the user (a hero's cast on one stack; morale,
  luck, Haste and Slow's walk, the count box's tint and the rest T7b-2). The viewer plays a hero's
  cast: its casting frames, a timed spell's line and animation (player 1), a resisted cast's
  sound, line and 78 beside the target, Magic Arrow's projectile and its animation over the struck
  stack's loop (players 7 and 2), their sounds; the outline cleared at the click, the spell book
  disabled from it, the aimed spell's status line. Harvested: the spells' sounds, animations and
  projectiles, the heroes' cast points; the spells' and heroes' names in the catalogue. `cast`,
  `resist`, `arrow` moved into `sequences.toml`: their timelines as recorded, `arrow`'s 168 frames
  exact, `cast`'s and `resist`'s but the count box's spell tint (T7b-2) and one frame of `cast`
  (the review list).
- **T7b-2a done 2026-10-21** ([log](E17-log.md#m3-t7b-2a--the-count-boxs-tint-haste-and-slows-walk-dispel-cure-disrupting-ray-2026-10-21),
  [design §4.3, §7.3, §7.16](E17-design.md)): T7b-2 split with the user (the parts without the
  original's rolls; morale, luck and the creatures' rolled animations T7b-2b). The count box's
  tint by its stack's spells (the spell records' sign harvested), Haste and Slow's walk time,
  Dispel, Cure and Disrupting Ray on one stack with their lines. `haste` and `dispel` recorded,
  played as recorded and their frames exact; `cast`'s and `resist`'s boxes exact.
- **T7b-2b-1 done 2026-10-21** ([log](E17-log.md#m3-t7b-2b-1--the-harness-forces-the-originals-rolls-2026-10-21),
  the runbook's "Forced rolls"): T7b-2b split with the user, who asked for the original's rolls
  reproduced at will first. `xdd.dll` redirects the calls of nine rolls (good and bad morale,
  fear, melee and shot luck, Death Stare, the Thunderbird's bolt, the Rust Dragon's acid, Poison)
  to a function that draws as the call would and returns a forced draw where the harness asks
  (`roll:<name>`, a sequence's `fail`); checked on good morale and fear in the original, four
  sequences replayed as recorded.
- **T7b-2b-2a done 2026-10-21** ([log](E17-log.md#m3-t7b-2b-2a--a-heros-luck-and-morale-on-the-originals-map-2026-10-21),
  the runbook's "Forced rolls"): at the user's word, luck made forceable. `h3-view situation`
  gives a scenario's hero the skill level behind each of its morale and luck `effects`
  (Leadership, Luck: the pack's level whose effects are that change alone); `h3mem` reads a
  stack's morale and luck. Checked in the original: the Marksmen at luck 3 and morale 3 as in
  the engine, a forced lucky shot (animation 18, its line), a failed one as the recorded `shot`.
- **T7b-2b-2b done 2026-10-21** ([log](E17-log.md#m3-t7b-2b-2b--morale-luck-and-fear-2026-10-21),
  design §7.16, §9.8): not split, at the user's word. Good luck (18) and good morale (20) over
  their stack, bad morale (30) and Fear (15) with its hit group beside them, their sounds (the
  asset tables' `[battle]` names, harvested), good morale's line after its animation and Fear's
  (GENRLTXT 729); a frozen stack never becomes the active one; the stack that acted last stays
  outlined through good morale and a freeze (found by the frame check). `luck`, `morale`, `bad`
  and `fear` recorded with forced rolls, their engine seeds searched: played as recorded, frames
  exact.
- **T7b-2b-3 done 2026-10-22** ([log](E17-log.md#m3-t7b-2b-3--the-creatures-strike-effects-2026-10-22),
  design §7.4, §7.16): not split, at the user's word. A melee strike's pending spells cast at its
  animation's end (Poison with the stack's hit, Acid Breath with the executable's own line, read
  through the catalogue's new `EXE:` source), then the strike's line and the striker's effect:
  Life Drain, Death Stare, the Thunderbird's bolt, the Rust Dragon's acid; a hero's Lightning
  Bolt; their sounds; a melee loop counts a defending stack's hit group; a cast's outline.
  `thunder`, `acid`, `stare` and `lightning` recorded with forced rolls, seeds matching the
  original's damage: played as recorded, frames as the original's but a landing flier's boxes.

- **T7c-1 done 2026-10-22** ([log](E17-log.md#m3-t7c-1--the-area-and-mass-spells-2026-10-22),
  design §7.16): T7c split with the user. The mass animation (Slow at Expert, Death Ripple and
  Destroy Undead's handlers, their lines), the area spells' animation at the hex and reactions,
  a creature's cast (the Faerie Dragon's group by the angle), `strike_animation` playing every
  stack on after its loop, the cast's tail (counts, outline), the mouse's highlight cleared as a
  cast starts, the book's empty status line. `mass` and `dragon` recorded (seed 174): played as
  recorded, frames as the original's but its redraw artifacts (the draw's log line listed).
- **T7c-2 done 2026-10-22** ([log](E17-log.md#m3-t7c-2--the-spell-objects-and-armageddon-2026-10-22),
  design §7.16): not split, at the user's word. The spell objects (Quicksand, Land Mine, Force
  Field, Fire Wall) placed by their animations at a hex, drawn in the obstacles' passes by the
  idle steps, gone with their vanish animations at a round's start before its line; Armageddon
  tiled over the battlefield with every stack's reaction, then its line. `obstacles` recorded
  (seed 517 487 719: the original's four pits): played as recorded, all 84 frames as the
  original's.
- **T7c-3 done 2026-10-22** ([log](E17-log.md#m3-t7c-3--raises-fire-shield-the-liches-cloud-and-chain-lightning-2026-10-22),
  design §7.16): not split, at the user's word. `raise_stack` (Resurrection, Animate Dead,
  Sacrifice's target, an Archangel, a Phoenix: Resurrection's 50 over the stack, a lying one's
  death group backwards, its line first), Fire Shield's return (11 over the shields with the
  striker's hit, its line), the Magogs' and the Liches' explosions (drawn plain, as spots), Chain
  Lightning (37 never shown, three rays at once for 100 ms each, 38 over every stack struck, its
  line); the rays' draws logged by the harness and kept for the frame check (the user's choice).
  `raise`, `creatures` and `bolt` moved from `casts.toml` (gone) into `sequences.toml`: played as
  recorded (seeds 9 and 566 for the original's damage); frames as the original's: `raise` 107,
  `bolt` 47 (every pixel of the rays), `creatures` 114 but the landing Archangels' redraw
  artifacts.
- **T7d done 2026-10-22** ([log](E17-log.md#m3-t7d--a-creatures-cast-its-drawing-bugs-fixed-2026-10-22),
  design §7.16): CLT-Q8's three drawing bugs fixed in what the client draws (a creature
  caster's group by the rows' angle, a missed caster standing at once, the mouse's stack
  unmarked at the action's start), the original's kept for the comparisons by
  `anim::Stage::bugs` (`h3-view sequences` only). Every sequence and situation drawn as before.
- **T9a done 2026-10-23** ([log](E17-log.md#m3-t9a--every-capture-afresh-2026-10-22-to-23)):
  every sequence (31) and situation (57) captured afresh, each as recorded (four after a rerun:
  the original's rolls and a step's lag, known traps); `kept/` refreshed, `drawn` the same.
- **T9b-1 done 2026-10-23** ([log](E17-log.md#m3-t9b-1--t9as-two-points-2026-10-23)): T9b
  split in five with the user (T9b-2 to T9b-4 the reviewer by area, T9b-5 the review list);
  `raise`'s varying `before` read as harmless (the frame check uses it in a cast only); a
  game's first screen waits through a display that dumps no 5-6-5 screen.
- **T9b done 2026-10-23** ([log](E17-log.md#m3-t9b--m3s-review-the-reviewers-and-the-review-list-2026-10-23)):
  three fresh reviewers in parallel (the user), 26 findings, 20 fixed (the timeline 7, the
  interface 5, the harness 9: a capture that differs no longer refreshes `kept/`), 5 queued;
  every frame drawn as before, the set and a sample of sequences as recorded; the review list
  accepted (the log); CLT-Q9 to Q17 answered (fixed, the original's for the comparisons only;
  Q15 listed).
- **T9c done 2026-10-23** ([log](E17-log.md#m3-t9c--the-next-milestone-the-lane-split-2026-10-23)):
  the user split the lane (CLT-Q18): sieges (M4) and then live play (M5) here, the adventure map
  in the new map-client lane (E24); the foundation and the harness common; T9d stays M3's last.

### The adventure lane's harness (H1, H2: its batch E3)

Taken ahead of M3's remainder on 2026-10-19 (the user): the adventure lane's E08 T22a waits on
it (`docs/rules/surveys/E08-battles.md` §13; the requests in the queue's inbox since 2026-10-13).

- **H1 done 2026-10-19** ([log](E17-log.md#h1--questions-the-difficulty-battles-from-the-map-2026-10-19),
  [design §6.3](E17-design.md)): `h3adv.py` answers a visit's questions by the record their text
  was written from (`h3lod.py`, the topmost dialog's text in memory), starts a game at a chosen
  difficulty, plays the battles a step opens (auto combat, a retreat or a surrender by side) with
  their stacks read at the start and the end, and records heroes' details, the players' resources
  before and after and a tile's monster from memory; `setup.sh` turns on the auto combat of
  creatures. The check recipe (`capture/batches/harness-check.toml`) plays each; batch 2 still
  reproduces.
- **H2 done 2026-10-19** ([log](E17-log.md#h2--the-underground-more-heroes-the-starts-reads-a-second-day-2026-10-19)):
  a `walk` onto either level (the view switched by the panel's button; shown on a shipped map),
  the hero list scrolled beyond five heroes, a level-up's skill chosen, the heroes off the map's
  experience, the date; the check recipe plays a second day.

### Later (refined when they come up)

**M4, sieges** (walls, towers, the Catapult, the moat; refined in M4 T0 after T9d), then **M5,
live play** (a battle played: hot seat in-process first, then against a server: the content
check, accounts; the server lane's notes in `NOTES.md`), in that order (the user, 2026-10-23,
T9c). Later: HD layouts, graphics options beyond `faithful`, the client's options (volumes),
video. The adventure map and the full game viewer: E24, the map-client lane.

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
| 9. The adventure map's object draw order (the user: neither they nor VCMI got it exactly right) | the map-client lane (E24) since the split (2026-10-23); M1 T5 read only the battle's |

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

## Review lists

Accepted by the user (M1 2026-10-13, M2 2026-10-14, M3 2026-10-23); in the log,
[`E17-log.md`](E17-log.md#accepted-review-lists-m1-m2-m3) (M1 and M2 moved there on 2026-10-20,
M3 at its review, T9b). The next milestone's list starts here.
