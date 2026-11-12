# E17 — Clients and asset pipeline

Status: **done** (2026-11-12, M7 T4): M0 (2026-10-12), M1 (2026-10-13), M2 (2026-10-14), M3
(2026-10-23), M4 (2026-10-27, T3c), M5 (2026-10-31, T8c), M6 (2026-11-10, T7c), M7 (2026-11-12,
T4); the adventure lane's harness H1 and H2 (2026-10-19). Each closed milestone's tasks with their
dates: `E17-archive.md` (with this line's earlier text). Refined 2026-10-12 (E17a, the asset
notes, done the same day). Decisions: [ADR-0020](../adr/0020-clients.md) (accepted at the M0
checkpoint, point 7 rewritten there), [ADR-0024](../adr/0024-client-window-audio.md). Working
detail (code readings, formats, layouts): `docs/epics/E17-design.md`; done tasks in full:
[`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset notes
are there), whose next epic is E27 (the game shell). Since T9c (2026-10-23) this epic is the
battle client's; the adventure map client is E24, the map-client lane's. Its open threads were
placed at its closure (below, "Placed at the closure").

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
  whole-archive reads; until it lands the adapter reads whole archives (ADR-0020 point 7). The
  reader landed (2026-10-19); the client's switch to it went to E27 at E17's closure.
- **Text through a string catalogue** (2026-10-13): every string the client shows is an entry of
  a key → string catalogue (nested JSON or similar), easy to translate; language packs swap. The
  original's text stays uncommitted (ADR-0009): its language comes from the user's own files
  through a committed catalogue of keys. **The packs are nested JSON** with i18next's conventions
  (2026-10-19, M3 T4b-1, over TOML and Fluent), and **a translation of the original's text is
  never committed**: a player may load a pack of their own, which the project neither ships nor
  hosts (CLT-Q7, 2026-10-19). An ADR-0020 amendment is drafted in `docs/lanes/client/adr-drafts/`.
- **Graphics options** (2026-10-13), eventually: per effect the original's way or a modern one
  (shadows first); `faithful` is the default and the reference. The draw list names what is
  drawn, the compositor decides how (E29 since E17's closure).
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
- **E17 ends with M7** (2026-11-10, M6 T7c): M7 builds M6's answers and the battle as a
  hosted entry, then closes the epic. Ranked play, the pause by agreement, the server browser
  and a login dialog leave E17: E27 (the game shell: the main menu and the screens outside
  play) comes first, then a refinement of how ranked play joins the client. The first ranked
  play launches through the website (E21's portal); once HotA's content is in, its additional
  screens are weighed for reuse. Where the password is typed is decided at that refinement.
- **The open threads at the closure** (2026-11-12, M7 T4): ranked play in the native client a
  backlog row of its own (E28), HD layouts and graphics options another (E29); the rest as the
  agent proposed ("Placed at the closure").

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

### Closed milestones M0 to M7

Moved to `E17-archive.md` (verbatim; the full records are in `E17-log.md`, the design in `E17-design.md`):

| Milestone | Done | Content |
|---|---|---|
| M0 — the workspace | 2026-10-12 | `clients/` as its own workspace, the gates (`clients/scripts/check.sh`, `clients.yml`), `h3-view gate`, ADR-0020 |
| M1 — assets and a battle still | 2026-10-13 | code readings, the decoders (palettes, PCX, DEF, MSK), the real-asset gate, a battle still |
| M2 — the battle replay viewer | 2026-10-14 | the capture harness, the window/input/audio choice (ADR-0024), the scene model, the replay viewer |
| M3 — fidelity | 2026-10-23 (T9d) | the scene over time completed, recordings against the original under Wine, the string catalogue, the lane split (CLT-Q18) |
| M4 — sieges | 2026-10-27 (T3c) | the siege still and over time read, drawn and recorded against the original (every town; the walls, the gate, the moat, the mines, Earthquake, the siege's mouse), the harness forcing rolls and reading the hovered item, every capture afresh |
| M5 — live play | 2026-10-31 (T8c) | hot seat in the window (the click's commands, the spell book, the hero's aim, the message box, Retreat and Surrender, the result window and its clip, Auto Combat, the tactics phase, the created stacks and their dissolves, the palette effects, the sounds' waits), then a seat on a server by a token and by an account; every capture afresh with the starting bonus Gold |
| M6 — the battle screen complete | 2026-11-10 (T7c) | a stack's info and the right button's popups (the stats dialog, the hero's and the turret's windows, the army info windows), the combat options dialog in effect (the speeds, the volumes, the grid and shadows, Auto Combat's options, Creature Info), M5's gaps (the book's refusals, the bot's casts and retreat question, Eagle Eye's message, the effects and their waits), the deck and the boats, the Demons' dissolve, the clips decoded as `BINKW32.DLL`, every capture afresh, a fresh review's 56 findings |
| M7 — the battle client's last steps | 2026-11-12 (T4) | M6's answers built (F6 to F8 on every turn, the slips, the reads and captures, the tools), the DEF button's own loop, the battle as an element a host's window runs (the library `h3-battle-client`, a whole game's battle from its record and played live through the host's game, the picture by a map's sex, the sound on the host's, the frame widened), a fresh review's 21 findings fixed, E17's open threads placed |
| H1, H2 — the adventure lane's harness | 2026-10-19 | questions answered, the difficulty, battles from the map, the underground, more heroes, the starts' reads, a second day (its batch E3) |

### Placed at the closure (M7 T4, 2026-11-12, with the user)

Every open thread of E17 has a home outside it (the section "Later" as it stood: `E17-archive.md`).

| Thread | Where |
|---|---|
| Ranked play in the native client (the ladders, the queue and accept step, ratings, leaderboards), the pause by agreement, the server browser, a login dialog in the window; the server lane's notes (`NOTES.md`, "The live client") | **E28** (the backlog; the user: a row of its own), refined after E27, the first ranked play through the website (E21) |
| HD layouts (the battlefield's origin and viewport), graphics options beyond `faithful` (`NOTES.md`, "Options and layouts") | **E29** (the backlog; the user: a row of its own), for both clients |
| The client's options (the volume settings 0x5996C0, a track resuming 0x59AA82, CLT-Q25's step one hex back as a gameplay option), video (the intro videos, the video archives: research item 4) | **E27** (its draft's Options and start) |
| CLT-Q2's switch: the archive set read by the LODs' tables at start and a batch's ranges | **E27** (its draft's start-up loading) |
| M6 T5e (the captured artifacts' message after Eagle Eye's, design §12.21), CLT-Q4's variants 0 and 1 recorded, `CLT-STUB-4` (Auto Combat in a live game's battle) | **with E24 M4 T5**: the client queue's inbox, the map client lane told |
| `CLT-STUB-5` (a hot-seat battle between two of the host's humans; M7 T4, R3-5) | **with E24 M5**: the client queue's inbox |
| CLT-Q63 (a), Q65 (b), Q69 (a), `CLT-STUB-1`: answered or registered, waiting on the battle and server lanes | the client queue's inbox (their inboxes hold the requests) |
| The battle screen's unbuilt gaps (`NOTES.md`, "The battle screen": the other pending spells of a strike, regeneration, Fire Wall's burn, Land Mine's blast and the others) | the client queue's inbox, taken when a recording needs one |
| The harness check's growth (a monster's offers, a pile's question, a walk below) | the adventure lane's inbox |
| `draw_frame_blend`'s encoding 3 (the map client lane's note of 2026-10-27) | informational: `QUEUE-ARCHIVE.md` (`h3_render::blit`'s *provisional* comment holds it) |
| The adventure map and the full game viewer | E24, the map-client lane (since 2026-10-23) |

## Research items from the asset notes

| Item | Where |
|---|---|
| 1. Archive precedence (SoD against AB), name lookup | M1 T1 (done: design §3.1) |
| 2. The off-colour pixels; the screen format | M1 T1 (done: design §3.2) |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | the result window's clips decoded (M5 T4b2b, M6 T6c); the rest E27 (placed at M7 T4) |
| 5. Capturing the original | M1 T8 (done: Wine in the sandbox, design §6); M2 T1 (done: the harness, §6.1) |
| 6. Per-expansion content and the ruleset profiles | M1 T4 |
| 7. The HD patch as a source of fixes and for HD | E29 (placed at M7 T4) |
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
  adventure map around the battle; a battle's own record does not carry it, so the viewer shows
  the picture of an empty surrounding there; a whole game's battle takes it from the wire (M7
  T3b1), unrecorded until a battle of E24 M4 T5 has such surroundings.
- **Whole archives read (CLT-Q2).** The table-only reader landed (2026-10-19), but the client
  still reads whole archives (212 MiB, about 0.2 s warm) until E27 switches to it.

## Review lists

Accepted by the user (M1 2026-10-13, M2 2026-10-14, M3 2026-10-23, M4 2026-10-27, M5
2026-10-31, M6 2026-11-10, M7 2026-11-12); in the log,
[`E17-log.md`](E17-log.md#accepted-review-lists-m1-to-m7) (M1 and M2 moved there on 2026-10-20,
each later one at its review: M3 T9b, M4 T3c, M5 T8c, M6 T7c, M7 T4). E17 is closed: no list
starts here.
