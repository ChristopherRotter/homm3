# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12), M1 (2026-10-13), M2 (2026-10-14), M3
(2026-10-23), M4 (2026-10-27, T3c), M5 (2026-10-31, T8c); M6 (the battle screen complete)
refined at M5's closure (2026-10-31); the adventure lane's harness H1 and H2 done (2026-10-19).
Each closed milestone's tasks with their dates: `E17-archive.md` (with this line's earlier
text). Refined 2026-10-12 (E17a, the asset notes, done the same day). Decisions:
[ADR-0020](../adr/0020-clients.md) (accepted at the M0 checkpoint, point 7 rewritten there).
Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`; done tasks in
full: [`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset
notes are there). Next: M6 T1 (`docs/lanes/client/QUEUE.md`). Since T9c (2026-10-23) this epic
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
- **M5's scope** (2026-10-27, M4 T3c): live play in two steps, hot seat in-process first (each
  seat a human or the Greedy bot), then a game on a server (a seat token, then accounts); the
  ladders, the pause and the server browser after M5. The battle recipes' starting bonus is set
  to Gold at M5's closure (M4's item 15), with that milestone's captures afresh.
- **Text is centred** (2026-10-31, M5 T8c, CLT-Q28): wherever the original sets a text off its
  centre, the client centres it in its own profile, the original's place kept for the
  comparisons only (`h3_render::Bugs`); a reading or a recording that finds such a slip from now
  on fixes it so (no question needed for it).
- **M6's scope** (2026-10-31, M5 T8c): the battle screen complete first (a stack's info, the
  right button's popups, the combat options dialog, M5's gaps, the inbox's waiting items), then
  M7 ranked play and the server browser; HD and graphics options after both.

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

### Closed milestones M0 to M5

Moved to `E17-archive.md` (verbatim; the full records are in `E17-log.md`, the design in `E17-design.md`):

| Milestone | Done | Content |
|---|---|---|
| M0 — the workspace | 2026-10-12 | `clients/` as its own workspace, the gates (`clients/scripts/check.sh`, `clients.yml`), `h3-view gate`, ADR-0020 |
| M1 — assets and a battle still | 2026-10-13 | code readings, the decoders (palettes, PCX, DEF, MSK), the real-asset gate, a battle still |
| M2 — the battle replay viewer | 2026-10-14 | the capture harness, the window/input/audio choice (ADR-0024), the scene model, the replay viewer |
| M3 — fidelity | 2026-10-23 (T9d) | the scene over time completed, recordings against the original under Wine, the string catalogue, the lane split (CLT-Q18) |
| M4 — sieges | 2026-10-27 (T3c) | the siege still and over time read, drawn and recorded against the original (every town; the walls, the gate, the moat, the mines, Earthquake, the siege's mouse), the harness forcing rolls and reading the hovered item, every capture afresh |
| M5 — live play | 2026-10-31 (T8c) | hot seat in the window (the click's commands, the spell book, the hero's aim, the message box, Retreat and Surrender, the result window and its clip, Auto Combat, the tactics phase, the created stacks and their dissolves, the palette effects, the sounds' waits), then a seat on a server by a token and by an account; every capture afresh with the starting bonus Gold |

### M6 — the battle screen complete

Refined at M5's closure (T8c, 2026-10-31) with the user; the tasks in
`docs/lanes/client/QUEUE.md`, each cut with the user at its start when it outgrows half a
context. What a played battle shows of the original's battle screen that M5 did not build: a
stack's and a turret's info, the right button's popups, the combat options dialog with its
settings in effect; with them M5's gaps (its review list's "not built") and the items that
waited in the inbox for a free slot. Ranked play and the server browser follow as M7.

- **T1, the answered slips** (done 2026-10-31): CLT-Q26, Q27 (1) and (2), Q28 with the text's
  centring everywhere (the box's text the readings' only one off its centre), Q33 and Q35 (its
  tune and clip with its text) fixed in the client's profile, the original's for the comparisons
  only; `drawn` all the same.
- **T2, a stack's info:** the left press on a stack opens its stats dialog (0x468860), the right
  button held over it shows it as a popup (0x4769C0), a turret's info (0x47A380); recorded.
- **T3, the right button's other popups:** a hero's, the panel's controls' help (0x472A50), the
  battlefield's other items; recorded.
- **T4, the combat options dialog:** the panel's Options read, drawn and in effect (the grid, the
  movement and cursor shadows, the combat speed, the Auto Combat options, the rest as read), kept
  in the client's settings; recorded.
- **T5, M5's gaps:** the book's refusal messages, the Auto Combat bot's casts and retreat
  question, Force Field's and Fire Wall's aim, Eagle Eye's message, Bloodlust's glow, the effects
  not drawn yet and their sounds' waits (M5's items 5, 9, 24, 42, 45, 79, 84), each as far as read.
- **T6, the inbox's waiting items:** the deck and the two boats as situations (CLT-Q5), `h3-bink`
  as `BINKW32.DLL`'s float transform (CLT-Q30), the Demons' dissolve, the dialog items through
  `h3_ui::paint`, the purity test's holes (R2-15), R4-2 and R4-9.
- **T7, M6's closure:** a fresh reviewer, every capture afresh, the review list, M7 refined.
- **Exit:** a played battle shows every dialog and popup of the original's battle screen, each
  passing against the original or with each difference a listed fix or a classified quirk; the
  combat options in effect.

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

After M6 (the battle screen, above): **M7, ranked play and the server browser** (the
ladders, the pause by agreement and the browser; the user, 2026-10-27 at M4 T3c and 2026-10-31
at M5 T8c; the server lane's notes in `NOTES.md`, "The live client": the original has no such
screens, so their look is new design for the review). Later: HD layouts, graphics options
beyond `faithful`, the client's options (volumes; a two-hex stack's step one hex back by the
mouse as a gameplay option, off by default: CLT-Q25), video. The adventure map and the full game
viewer: E24, the map-client lane.

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

Accepted by the user (M1 2026-10-13, M2 2026-10-14, M3 2026-10-23, M4 2026-10-27, M5
2026-10-31); in the log, [`E17-log.md`](E17-log.md#accepted-review-lists-m1-to-m5) (M1 and M2
moved there on 2026-10-20, M3 at its review, T9b, M4 at its review, T3c, M5 at its review, T8c).
The next milestone's list starts here.

**M6** (for its closure, T7):

1. The profile's switches of the answered slips (T1): `book::Memory::new` (a session's memory
   in its profile, the default the client's), `message::layout`'s `(player, bugs)` and
   `message::framed`, `results::Sources::bugs`, `sound::end_tune`'s `bugs`; the comparisons
   (`render`, `sequences`, the book's stills) build the original's.
2. The window's boxes framed by the acting side (T1, CLT-Q28 (2)): the controller of the turn the
   box's time falls in (`Timeline::controllers`, as the panel's), the tactics phase's side for its
   message; the result window keeps the viewing player's colours (no side's question).
3. CLT-Q35's tune and clip (T1): the user answered for the text; the agent took the tune and the
   clip of the same branch with it, so the window that says the defender's army fell plays
   `LoseCombat` and its clip, not the fleeing ones.
4. The centring directive's survey (T1): the battle's readings (design §7.17 to §7.24, §9) set
   one text off its centre, the message box's; the text drawer's halves truncated (an odd
   difference a pixel left) are the integer centre, not a slip.
