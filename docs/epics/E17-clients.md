# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12), M1 (2026-10-13), M2 (2026-10-14), M3
(2026-10-23), M4 (2026-10-27, T3c); M5 (live play) refined at M4's closure (2026-10-27); the
adventure lane's harness H1 and H2 done (2026-10-19). Each closed milestone's tasks with their
dates: `E17-archive.md` (with this line's earlier text). Refined 2026-10-12 (E17a, the asset
notes, done the same day). Decisions: [ADR-0020](../adr/0020-clients.md) (accepted at the M0
checkpoint, point 7 rewritten there). Working detail (code readings, formats, layouts):
`docs/epics/E17-design.md`; done tasks in full: [`E17-log.md`](E17-log.md). Lane: client
(`docs/lanes/client/HANDOFF.md`; the user's asset notes are there). Next: M5 T3b (the hero's
aim; `docs/lanes/client/QUEUE.md`). Since T9c (2026-10-23) this epic is the battle
client's; the adventure map client is E24, the map-client lane's.

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

### Closed milestones M0 to M4

Moved to `E17-archive.md` (verbatim; the full records are in `E17-log.md`, the design in `E17-design.md`):

| Milestone | Done | Content |
|---|---|---|
| M0 — the workspace | 2026-10-12 | `clients/` as its own workspace, the gates (`clients/scripts/check.sh`, `clients.yml`), `h3-view gate`, ADR-0020 |
| M1 — assets and a battle still | 2026-10-13 | code readings, the decoders (palettes, PCX, DEF, MSK), the real-asset gate, a battle still |
| M2 — the battle replay viewer | 2026-10-14 | the capture harness, the window/input/audio choice (ADR-0024), the scene model, the replay viewer |
| M3 — fidelity | 2026-10-23 (T9d) | the scene over time completed, recordings against the original under Wine, the string catalogue, the lane split (CLT-Q18) |
| M4 — sieges | 2026-10-27 (T3c) | the siege still and over time read, drawn and recorded against the original (every town; the walls, the gate, the moat, the mines, Earthquake, the siege's mouse), the harness forcing rolls and reading the hovered item, every capture afresh |

### M5 — live play

Refined at M4's closure (T3c, 2026-10-27) with the user; the tasks in
`docs/lanes/client/QUEUE.md`. A battle played in the window: hot seat in-process first (each seat
a human or the viewer's Greedy bot), then a game on a server (by a seat token, then accounts).
The viewer plays records whose commands are all known before the first frame; live play adds
commands as they come, takes them from the mouse and the keys, and draws what a played battle
shows that the recordings did not need: the spell book, the hero's aim, the battle's dialogs,
the stacks created mid-battle.

- **T1a, a frame without the rescan** (R2-7; done 2026-10-27): `anim::Playhead` reads the
  timeline forward, a frame costing what changed since the one before (0.3 to 1 µs, against
  `Moment::at`'s 1.6 to 7 µs over `play`'s 82 seconds and 306 µs half an hour after its end; the
  2.3 ms a frame was the composing); `play`, `window` and `sequences` draw through it.
- **T1b, the timeline that grows** (T1 cut in two, the user; done 2026-10-27): `anim::Growing`
  takes the commands as they arrive (its lists only appended to); `play` and the window fight a
  scenario as they play it (`live.rs`: the session in-process, both seats held, the bot's
  command asked for when a frame reaches the timeline's end), the record, the panel, the mouse
  and the batch growing with it; `play`'s output byte for byte as fought at once.
- **T2, hot seat: the player's commands** (cut in two by the user, 2026-10-27; the Faerie
  Dragon's cast moved to T3, its key entering the hero's aim):
  - **T2a, the click's command** (done 2026-10-27): `mouse::click` and `button` (a walk, a
    strike from the slot's from-hex, a shot, a heal, a wall shot, Wait and Defend), read (design
    §9.11: the input queue emptied at every turn's start); CLT-Q24's switch; scripted presses
    over every scenario's turns give the engine's legal commands but a two-hex stack's step one
    hex back (CLT-Q25: the original's copied).
  - **T2b, hot seat in the window** (done 2026-10-27; whole, the user): `--human` (the sides a
    human plays here, the battle's human players); the battle waiting at its timeline's end
    (`Growing::waiting`), a press's or a button's command through the session, input made while
    the battle takes none dropped; the screen showing the side to act's hidden objects (every
    side's events, `Timeline::screen`); unattended runs pressing the bot's choices (design
    §9.12).
- **T3, the spell book and the hero's aim** (cut in two by the user, 2026-10-27):
  - **T3a, the spell book** (done 2026-10-27): read (design §7.17: the dialog's items, the page,
    the keys, the move's hint and cursor, the shadow), drawn (`h3_view::book`, `ui`'s DEF items,
    texts alone and areas) and used in the window (its button or C, a spell without a target
    cast at once); its stills (`book`, `book-page`, `book-fire`, `book-hint`) exact; CLT-Q26.
  - **T3b, the hero's aim** (done 2026-10-28; whole, the user): read (design §7.18: the aim's
    loop and cursor, the cast's line and the refusals, the area's marks, the spell cursor's
    frames, the callers' ends, the cast's start), built (`mouse::aim`, the aim in `track`, the
    window's aim by the book or a Faerie Dragon's F, Escape or the right button giving it up,
    unattended casts pressed through the book and the aim); `spell-aim` recorded; `cast`'s 71
    pixels found to be the book's closing, a redraw artifact (listed); CLT-Q27.
- **T4, the battle's dialogs** (cut in three by the user, 2026-10-28): read, drawn, recorded.
  - **T4a, the message box, Retreat and Surrender** (done 2026-10-28): read (design §7.19: the
    box's layout, frame, items and input, the text tables' quotes), built (`h3_view::message`,
    the window's and the stills' boxes: Retreat's and Surrender's questions and refusal, the
    book's refusal and description, "affects no one"); six stills exact; CLT-Q28.
  - **T4b, the result window** (cut in two by the user, 2026-10-28):
    - **T4b1, the window without its clip** (done 2026-10-28): read (design §7.20: the items, a
      hero-less side's face, the losses, the text, the experience, the handler), built
      (`h3_view::result`, a DEF item's mirror, the heroes' portraits; the window's result opened
      as the battle is decided, its OK closing the battle's screen); `result-retreat`,
      `result-surrender` and `end`'s last frame exact, `defended`'s but 122 artifacts.
    - **T4b2, the clip** (cut in two by the user, 2026-10-28; a port of FFmpeg's decoder, the
      user's choice, CLT-Q29):
      - **T4b2a, the decoder** (done 2026-10-28): `h3-bink` (the Bink file, the video of revision
        `b`), `h3_assets::vid`, `h3-view video` and `bink-check.sh`; the ten clips and eight
        synthetic samples as FFmpeg decodes them, byte for byte.
      - **T4b2b, the clip in the window** (done 2026-10-28; design §7.21): the playback read
        (0x597570 to 0x44DAA0, the archives' lookup), the timing and the colours found by
        running `BINKW32.DLL` as a black box (`binkprobe.exe`) and in its tables (the user's
        leave), drawn in the window, the stills and the sequences (`h3_view::clip`,
        `Draw::Video`); the harness given the clips and the DLL's clock; CLT-Q30 (the DLL's float
        DCT: listed, exact later), CLT-Q31 (two timing slips: fixed, the original's for the
        comparisons); `result-*` and `end` exact but the DCT's pixels, `defended` as before plus
        them.
  - **T4c, Auto Combat and the tactics phase** (cut in two by the user, 2026-10-28):
    - **T4c1, Auto Combat** (done 2026-10-28; design §7.22): read (the switch, the stop by the
      input queue's read at a turn's start, the panel dimmed with its button selected, the
      mouse), built (`h3_view::auto`, the panel's `Autoplay`, the window's switch and stop, the
      bot playing the human's side, `--auto`); `auto` and `auto-stop` recorded, their panels and
      cursors exact.
    - **T4c2, the tactics phase** (cut in two by the user, 2026-10-29):
      - **T4c2a, the phase drawn** (done 2026-10-29; design §7.23): read (the message at the
        phase's start, its panel 0x46C050, the panel built for the defender, the movement
        shade's overlay and search, the cursor), built (`h3_view::tactics`, `Draw::Shade`, the
        layout's `[tactics_panel]`, the message before the first turn in the stills and the
        window); seven stills exact; CLT-Q32.
      - **T4c2b, the phase played** (done 2026-10-29, kept whole by the user; design §7.24):
        read (the placement as the walk, Next Creature, Start Combat and the panels' swap, the
        shade over the turns, the computer's phase, Auto Combat in it, the keys), built (the
        engine's placement steps, `cross-lane`; the buttons' commands, `Context::Tactics`, the
        window's panel shown, `tactics::shade_at`); `tactics-play` recorded.
- **T5, what a played battle shows** (cut in two by the user, 2026-10-29):
  - **T5a, the dissolves and the created stacks** (done 2026-10-29; design §7.25): read (the
    blend, its rectangle, the stacks removed without a corpse, the created stacks, Clone's
    place and slide, Teleport's), built (`Screen::dissolve_rect`, `anim::created`, the window's
    notices, the engine's `NotCreated`, `cross-lane`); `teleport`, `clone`, `summon`, `demons`
    recorded, `sacrifice` exact; the Demons' back buffer a slip not copied; CLT-Q33.
  - **T5b, the palette effects, the panel by the turn's controller, the sounds' waits** (cut in
    two by the user, 2026-10-29):
    - **T5b1, the palette effects** (done 2026-10-29; design §7.26): read (`draw_battle_stack`'s
      chain, the recolour, Stone Gaze's glow, the petrified look, the HSV round trip, the glows'
      redraws, Stone Gaze's line), built (`h3_render::recolour`, the timeline's `LookKey`s,
      `Draw::Creature`'s `effect`); `bloodlust` and `stone` recorded, their glows and looks exact;
      `clone`'s colours exact; CLT-Q34.
    - **T5b2, the panel and the sounds' waits** (done 2026-10-29, kept whole by the user;
      design §8.3, §9.12): read (the acting side by the controller, `wait_for_sample` and
      `sample_wait` at their 30 battle sites), built (`Timeline::controllers`,
      `Panels::with_controllers`, `sound::Lengths`, the timeline's waits, the window's lengths;
      the harness's `sound` runs); `hypnotize`, `summon-turn` and six `*-sound` sequences
      recorded; CLT-Q34 answered (a).
- **T6, a game on a server** (cut in two by the user, 2026-10-30: the part that needs no
  original data first; design §11):
  - **T6a, the connection, headless** (done 2026-10-30; design §11.1): the user's dependency
    (the server's stack, blocking on the I/O thread only: `tungstenite`, `rustls` with
    `rustls-graviola` and the OS roots, `ed25519-dalek`, `getrandom`); `h3_view::remote` (the
    hello, the welcome's checks, the pins, a join by token or a game created, the record fetched
    and grown, commands, the content check), the I/O thread (`net`), `h3-view connect`; tested
    over loopback (`ws://` and `wss://`, a drop and a re-join) and against `h3-server`.
  - **T6b, the window over a server** (cut in two by the user, 2026-10-30):
    - **T6b1, the window plays a seat** (done 2026-10-30; design §11.2): `h3-view window
      --server`, the battle growing by what the server sends (polled at each frame), the turn
      from `YourTurn` with the aims' targets by `Query`, the seat's battle derived
      (`h3_view::remote::seen`), both content outcomes, a drop's re-join keeping the timeline,
      `remote_turn` (the seat's player the one human here); two windows on a local `h3-server`
      played a battle to its end, every frame exact.
    - **T6b2, the record that opens mid-battle** (done 2026-10-30, kept whole by the user;
      design §11.3): the snapshot's dead among the corpses (R2-8: raised as recorded), the box
      by the timeline's living set, the moat's strip by the fixed rows (R1-3); a re-join's
      record extends the one taken or opens afresh (`remote::rejoined`): a new timeline; a join
      mid-battle and a restart of a local `h3-server` played, every frame exact.
- **T7, accounts** (done 2026-10-30, kept whole by the user; design §11.4): `argon2` the
  engine's (the user's choice), the password from standard input shown as typed (the user's
  choice); `h3-view account` derives and keeps the key (`remote::Accounts`), the test vector
  reproduced; `Client::login` with the re-key from an earlier key due at the server after a
  password change; seats by account; `remote::farewell`; played logged in on a local
  `h3-server`, headless and in two windows to the result window, seats re-taken by account
  after drops, every frame exact.
- **T8, M5's closure:** a fresh reviewer, the recipes' starting bonus set to Gold (M4's item 15),
  every capture afresh, the review list, the next milestone refined. Cut by the user into
  concurrent fractions (QUEUE); **T8a done:** the harness's two instances, the bonus Gold,
  reviewers R1, R3, R4 and their fixes, all stills and sequences 0 to 27 as recorded. T8b cut
  in two by the user; **T8b1 done:** sequences 28 to 64 as recorded, reviewers R7 (the harness)
  and R2a/R2b (the decoder, the clip) and their fixes.
- **Exit:** a battle played to its end in the window, in hot seat and on a server; the new
  screens (the book, the aim, the dialogs, the created stacks) pass against the original, or each
  difference is a listed fix or a classified quirk.

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

After M5 (live play, above): the ladders, the pause by agreement and the server browser (the
user, 2026-10-27, M4 T3c; the server lane's notes in `NOTES.md`, "The live client"). Later: HD
layouts, graphics options beyond `faithful`, the client's options (volumes; a two-hex stack's
step one hex back by the mouse as a gameplay option, off by default: CLT-Q25), video. The
adventure map and the full game viewer: E24, the map-client lane.

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

Accepted by the user (M1 2026-10-13, M2 2026-10-14, M3 2026-10-23, M4 2026-10-27); in the log,
[`E17-log.md`](E17-log.md#accepted-review-lists-m1-to-m4) (M1 and M2 moved there on 2026-10-20,
M3 at its review, T9b, M4 at its review, T3c). The next milestone's list starts here.

**M5** (for its closure, T8):

1. The sides a human plays in the window are the battle's human players, the bot's the
   computer's (T2b, `--human`; design §9.12): the engine's computer-only rules (BAT-QUIRK-27's
   let-off) apply to the bot's side; without `--human` the scenario's `human` stays.
2. The spell book's costs and school levels in the client (T3a, `spellbook::hero`; design
   §7.17): the cost the ruleset's at the mastery plus the observation's `cost_change` (a copy of
   the engine's BAT-MAG-054 formula; a public engine function or an observation field would
   replace it), the hero's school levels from the book's spells of one school (the observation
   gives masteries by spell: a school none of them names counts as none, which can put Magic
   Arrow under another school than the original on the tab of all schools).
3. The capture set's situations take actions (T3a, `actions`: `key:`, `click:`, `move:` after the
   mouse's move, at the frozen clock) and the harness sets "Animate SpellBook" off (the page
   turn's videos, which the client does not play): the stills of a dialog.
4. The layout format's new kinds (T3a: `def`, `label`, `area`, an entry's `repeat`, a dialog's
   `shadow`; the `[book]` section) and the catalogue's (a family's `list`, `contexts`, `raw`).
5. The aim's derivations in the client (T3b, design §7.18): a refusal's reason by elimination
   (any refusal of the engine's that Anti-Magic and the polarity do not explain reads as the
   immunity: the engine's tests are not public), an area's marks on every living stack of it (the
   original's only on those the spell may affect), Berserk's area cut at the side columns as the
   other areas'; the aims of Force Field, Fire Wall, Teleport and Sacrifice not built (the book
   closes on them).
6. The window's keys and the adapter (T3b): F aims a Faerie Dragon's spell, full screen moved to
   F11; the right button comes as a key, `h3_sdl::Key::RightButton` (a common path, additive).
7. The harness (T3b): the action `aim:<k>:<n>`, and the spell cursor's 100 ms grid (0x69CA1C)
   set at a sequence's opening with the poll's: it changes only frames that show a type-3 cursor
   for a grid point, which no capture before `spell-aim` did, so the captures were not all taken
   again (M5's closure takes them).
8. The message box's formats (T4a, design §7.19): the layout's `[message]` (the box's assets,
   fonts and cursor by name; the box itself laid out in code from its text, `h3_view::message`),
   a picture item drawn at most its size (`ui::Picture::size`, the mouse's area until then) and a
   DEF item in a player's colours (`Draw::Button`'s `player`).
9. The box's derivations in the client (T4a): the viewing player as the attacker's when a human
   plays it here, else the defender's (CLT-Q28 (2)); Surrender enabled when the side's own view
   prices it (the engine's terms), Retreat when the engine lists it; not built, as the engine
   refuses those casts without telling why: `hero_book_button`'s refusals (an anti-magic
   garrison, the Orb of Inhibition, Cursed Ground, Recanter's Cloak) and the aim's entry's other
   messages (Sacrifice, Remove Obstacle, Clone's and the Summons' limits, Earthquake outside a
   siege, Land Mine against natives).
10. The catalogue makes each run of quotes in a table's field one, as the executable reads its
    tables (T4a; `text::pack`): `h3_formats::text` keeps them (the battle lane's inbox).
11. The window while a box is open (T4a): every key but the viewer's own goes to the box (Escape
    is its Cancel's, never the window's quit); unattended runs' `--leave retreat|surrender`.
12. The result window's formats (T4b1, design §7.20): the layout's `[result]` (its assets by
    name; the window laid out in code, `h3_view::result`), the asset tables' heroes' `portrait`
    (harvested), a DEF item's `mirrored` (and `Draw::Button`'s), the items the executable gives
    the id 200 given ids of their own.
13. The window's derivations in the client (T4b1): the experience a copy of `battle_experience`
    0x46A350 over the engine's `BattleOutcome`, without Learning (the client's heroes have none;
    `h3-game`'s is private: a public one would replace it), a hero's portrait its type's (a
    map's own portrait not known), a hero-less side's face from the reading only (the harness's
    maps give both sides a hero, so no capture shows one).
14. The window's battle ends with the result's OK (T4b1): the window's run ends there, as the
    original leaves the battle's screen.
15. `h3-bink` (T4b2a): a new pure crate, a port of FFmpeg 6.1.1's revision-`b` decoder (CLT-Q29)
    under the GPL-2.0-or-later with FFmpeg's copyright lines; it indexes its fixed tables and
    blocks directly where the index is bounded by construction (no `clippy::indexing_slicing`
    denial, which `h3-assets` has), the planes and bundles only through checked reads and
    writes; frames above 800 × 600 refused.
16. The decoder's buffers (T4b2a): the block grid's rows below the picture keep the frame
    before's, as FFmpeg's decoder when each frame is released (its command line, holding the
    frame before, loses them: three clips differ there in a few pixels; RAD's decoder's
    behaviour is T4b2b's captures' to show).
17. The decoder's check (T4b2a): `clients/scripts/bink-check.sh` takes its reference from PyAV
    (`pip install av`: BSD-3-Clause, FFmpeg's libraries), a tool of the sandbox's, no crate's
    dependency; the synthetic samples 256 pixels wide, as the clips (FFmpeg pads narrower rows).
18. `VIDEO.VID`'s entries sized by the next larger offset (T4b2a, `h3_assets::vid`): no size is
    stored, so entries sharing an offset read the same bytes.
19. The harness's clip tools (T4b2b): `binkprobe.exe` (our code, built by `setup.sh` with
    mingw-w64) runs the game's `BINKW32.DLL` outside the game under Wine as a black box, its
    clock import ours; `xdd.dll` points the DLL's `timeGetTime` import at the virtual clock
    (its reads not the game's); the run directory's `VIDEO.VID` is the user's
    `battle-videos.vid`.
20. The clip's draw (T4b2b): `Draw::Video` names the start and the loop and carries the time
    since the window opened, the compositor picking the frame from the batch's decoded clips
    (`clip::shown`); the batch holds a clip decoded whole, 5-6-5 (`LoseCstl` 10.7 MB); the
    window draws at the clip's decodes (`clip::decodes`, `Battle::clip_times`), the draw list
    changing every millisecond.
21. The video archives in the binary (T4b2b, `videos.rs`): only the directories read
    (`h3_assets::vid::directory`, `directory_bytes`), an entry when a batch takes it; an
    installation copy without `VIDEO.VID` takes `original-data/battle-videos.vid` (a cloud
    sandbox's), one without either shows the slot bare.
22. The result window's clip in the formats (T4b2b): the layout's `[result]` `clip` (the slot's
    corner), `results::window` returning its tune (the clip by the same branches; a test keeps
    it equal to `sound::end_tune`'s).
23. Not modelled (T4b2b): in the original a redraw of the window while its clip stands at its
    first frame (a press over the window in its first 133 ms) delays the clip a frame more; the
    client's own timing (CLT-Q31) has no such delay, the comparisons' none either.
24. Auto Combat's settings (T4c1): the client's bot plays every stack of the human's side, as the
    original with all its Auto Combat settings on (`Combat Auto Creatures`, the spells, the
    Catapult, the Ballista, the tent); no options dialog yet. The bot casts nothing and never
    leaves, so the computer's casts for the human's hero and its retreat question (design §7.22)
    are not built.
25. The window's stop (T4c1): every key but the window's own (the table's `Window` commands: Esc
    quits even while Auto Combat plays) and every press; the turn after a stop is decided as it
    starts (`Battle::play_next`), the outline of a human's turn it stops at put back
    (`Growing::outline_waiting`).
26. The formats (T4c1): `Input::Auto` and `Input::Stop` in `h3_view::mouse::track`, the
    sequences' `key:a` read by the original's key table, a sequence's end after its last `at:`
    action (`hover` compared to 1 500 ms, as the harness records it).
27. `--auto <n>` (T4c1): an unattended run switches Auto Combat on at the human's n-th turn and
    stops it by a press two commands later; the battle checked against the bot's at once.
28. The tactics phase's message in every battle (T4c2a; design §7.23): the original shows it once
    a game to each player (its record's +0x88); a battle in the client is a game of its own, so
    every battle opening with a human's phase shows it (`tactics::told`).
29. The shade's search in the client (T4c2a, `tactics::shade`): the engine's legal placements
    are not used; the client searches the observation (obstacles, stacks, two-hex fronts) as
    `battle_pathfinder` does, a flyer over any run of hexes it does not fit (the original's:
    fewer than its speed in a row, 0x4B39E7); a test holds the engine's placements in it
    (`tactics.toml`).
30. The formats (T4c2a): the layout's `cell_shade` and `[tactics_panel]` (the base's seven
    buttons listed again), `scene::Draw::Shade` (a picture's indices halving the screen,
    `h3_render::blit::darken_under`), `Moment::shade` (set by its viewers, not the timeline:
    the phase's later turns T4c2b), `Panels::before`, `Moment::before_first_turn`, the
    stills' `Still::opening`, `arrow` and `over`.
31. The panel built for the defender (T4c2a; 0x462D24): before, *provisional*, for the opening's
    acting side; the two differ on a computer attacker's opening against a human defender (the
    picture's colours until the human's first turn), which no capture shows.
32. The phase's keys (T4c2b; design §7.24): Next Creature's Space and Start Combat's Enter in a
    context of their own (`keys::Context::Tactics`), which the window asks before the battle's
    while the phase's panel shows (the original's disabled Defend passes Space on); a user's
    file binds `next-creature` and `start-combat` apart from the battle's.
33. The window's bot in the tactics phase (T4c2b): it ends any phase at once, a computer's and a
    human's under Auto Combat, where the original's computer places the stacks and ends its
    phase by its bound (BAT-AI-023); *provisional* with the bot (M2's review list: not the
    original's computer).
34. The formats (T4c2b): `tactics::shade_at` (a record's turns by its log's actions and turns),
    `Panels::turns`, `now`, `enabled`, `panel::turns`' moves, `Battle::panel_at` and
    `in_tactics`, `pointer::inputs`' `told` (a sequence's opening `key:Return`).
35. `EndTactics` in the viewer's timeline (T4c2b): a command without frames, its action's start
    (the pointer, the log settled) where the original's Start Combat runs no action; round 1's
    first turn starts in the same millisecond and does the same, so nothing shows it.
36. A created stack's numbers (T5a; design §7.25): a record holds the first snapshot only, so
    `created_stack` takes the creature, count and hex from the event, the body from the ruleset
    (`Casts::bodies`; a clone its original's) and leaves the rest at 0 (attack, defense, damage,
    speed: no drawing reads them; the mouse's status line over one in a record's playback names
    nothing).
37. A dissolve's moments (T5a): its changes drawn at its start and shown 1 ms later
    (`DissolveKey`), the moment at the start the saved picture; the battlefield objects with more
    than one frame left out of the rectangle (the original's `battle_refresh_creatures` widens it
    by them: no recorded dissolve has one).
38. The formats (T5a): `Timeline::dissolves`, `created`, `notices`, `Moment::created` and
    `dissolve`, `Draw::Dissolve` (its `measured` items), `Logged::Created` and `Summoned`,
    `Name::Creature`, `Casts::bodies`, the keys `battle.log.summoned` and
    `battle.clone.no-place`; `h3_render::Screen::dissolve_rect` in the foundation.
39. The clone's message in the window (T5a): a notice opens its box at its time and holds the
    window's clock there until it closes (the original's modal box inside the cast); not recorded
    (a Clone without a place needs every ring around a crowded original blocked).
40. The palette effects' cost (T5b1; design §7.26): each stack with one has its creature's
    palette recoloured at every frame drawn (246 entries, about 220 µs in a release build), not
    cached (*provisional*: a cache by DEF and effect would compute each once).
41. The formats (T5b1): `h3_render::recolour` (`Effect`, `apply`, `FIRST`) in the foundation
    (additive), `Timeline::looks` and `LookKey`, `Moment::looks`, `Draw::Creature`'s `effect`,
    `Logged::Petrified`, the key `battle.log.petrified`; the harness's roll `stone_gaze` (`xdd.c`'s
    sites, `h3drive.py`'s `ROLLS`: a redirected call draws as the original's, so no other capture
    changes).
42. Not built (T5b1): an Ogre Magi's Bloodlust (a creature's cast other than an area spell takes
    effect at once, M3 T7c-3): it glows nothing.
43. A sample's length (T5b2; design §8.3): its `fact` frames at its rate, rounded up; on a sound
    card Miles' mixer ends it, perhaps a few ms later. The harness's samples end so too, so the
    recordings confirm where the waits are and how long, not the mixer's slack.
44. A wait on a handle another sample took meanwhile (T5b2): the original would wait for that
    sample; the viewer waits for its own (no recorded case: every channel of a category busy, or
    the cue's sample over and the handle given again before the wait).
45. Not built (T5b2): the waits of the effects the viewer does not draw yet (the Wraiths' mana
    drain, regeneration, Poison's round damage, Death Blow, Ice Bolt's explosion, the Familiars'
    channel; design §8.3's list); the effects' switch (E) does not stop the waits (the original's
    volume 0 would: a setting, later).
46. The window makes its battle twice when its sound opens (T5b2, `Ground::play_sounds`): the
    library loads after the window opened; a live battle's opening plays twice (cheap, seeded).
47. The harness's sound (T5b2, the runbook's "Sound"): Wine's ALSA driver over the null sink and
    `xdd.dll`'s Miles hooks (nine imports): an emulation of a sample's end, not Miles' own; only
    the main thread's clock reads count in a `sound` run (the silent runs as before).
48. The connection's formats (T6a; design §11.1): the pins file (`address key-hex` a line,
    sorted; default `pins.txt` in the working directory, as `keys.toml`: a per-user place
    later), `remote::Client`'s notes and faults, `connect`'s options and its record's default
    path.
49. The record fetched at every join (T6a): one path for a join, a re-join after a drop and one
    after a restart, at the cost of a record's bytes (512 KiB at most by the server's default)
    per join; opened at its first snapshot.
50. The I/O thread polls (T6a): its read waits 5 ms, then it looks at the queue (a command waits
    up to 5 ms to leave); messages up to 64 MiB.
51. The plain player (T6a, `connect::plain`): a test's player, not an opponent (ends a tactics
    phase, shoots, strikes, else walks toward the enemy's edge by the column); the window's
    player is the human.
52. The seat's battle derived on a server (T6b1, `remote::seen`): the panel's turns, the moves,
    teleports and creations, the timeline's counts and deaths, the heroes' mana by the casts
    (the enemy hero's casts cost nothing: its book is unknown), the gate; a shooter's shots, the
    stacks' effects, the walls' hexes and parts stay the opening's (hints only: the commands are
    the server's).
53. The other seat's turn (T6b1): outlined from its start as a human's, its command's action
    from the poll it arrived at; the panel's buttons as on a computer's turn (the seat's player
    the one human here, so the end's window and tune by the seat: `remote_turn`); the original's
    network turn not read.
54. The content note (T6b1): in the window's title (`Different`: the client's own ruleset for
    the book and the result window, no aim areas); `Absent` stops the loading with a message.
55. The window on a server (T6b1): the loading waits for the record (30 s between messages);
    the window's input wait at most 10 ms; Auto Combat off; a dropped connection dialed again
    after a second; the unattended player the plain one, casting first the first spell the server
    lets the hero cast at its first target (the book and the aim pressed).
56. A corpse at a record's opening (T6b2, R2-8): its death group's last frame facing its side's
    way (the snapshot holds no facing), its record kept; a stack the battle made after the
    war machines by id. The corpses on a hex by the snapshot's order of death, then the
    timeline's by id (a stack raised and killed again keeps its first place).
57. The moat's strip (T6b2, R1-3): by the fixed rows 0x63BCE8, the Fortress's second row
    0x63BCF4 found by the record's moat holding one of its hexes (no Fortress flag on the wire).
58. A seat's window starts where its record stands (T6b2): a join mid-battle does not play the
    history again (the log still lists it); the start sound only from the battle's start.
59. A new timeline (T6b2): a record fetched at a re-join whose numbers do not go on from the
    one taken (a restart; a gap past the server's bound) opens the battle again from its
    snapshot, the window's clock, inputs and presses reset; the sounds stay keyed by the first
    opening's stacks.
60. Re-dialing (T6b2): once a second until a server answers, however long; the drop said once
    until a welcome.
61. The accounts file (T7): `accounts.txt` in the working directory by default (as `pins.txt`),
    a line per key (`<login> key|old <seed> [<address>...]`), the seed in the clear, the file
    made owner-only on Unix (Windows: the folder's permissions); no system keychain.
62. The password (T7, the user's choice): from standard input, a line shown as typed, its prompt
    only on a terminal; derived once per device by `h3-view account`; no dialog in the window
    (the original has none to copy; the server browser's milestone may want one).
63. A password change (T7): the key so far due at every server pinned then (an older key's
    servers left to it), re-keyed at the next login there; an earlier key due nowhere is
    forgotten; a server never dialed again keeps it in the file (its line edited by hand).
64. The final goodbyes (T7, `remote::farewell`): `Rekeyed`, `CommandLimit`, `Abandoned`,
    `Replaced` and `Protocol` end a seat's run with their reason (the window closes, the reason
    on standard error); `Replaced` re-dialed before T7, which made two connections of a seat take
    it from each other in turn. Other goodbyes re-dial as a drop.
65. A login refused ends the run (no second attempt, no prompt); `--register`'s display is sent
    until the run's first `LoggedIn`, not at its re-dials (T8a, R1-2: the server counts each as
    an account change, 20 an hour).
66. The harness's two instances (T8a): `H3_RUN`, `H3_DISPLAY`, `WINEPREFIX` (instance 2
    `original-data/wine2`, `:96`, `~/.wine-h3b`), `h3mem.game_pid` by the prefix; the battle
    recipes' starting bonus Gold for every player (`gold_bonus`, M4's item 15), the adventure
    batches and the map's stills left random.
67. The client's pace (T8a, R1-1): at most 10 messages in any second of the client's clock
    (`remote::Pace`; the server's limit, 20 by default, is not on the wire), and `RateLimited` a
    final goodbye.
68. Silence and waits (T8a, R1-3): the dial and each handshake read at most 10 s (`DIAL_WAIT`),
    any frame (pings too) heard, 60 s of silence a drop (`SILENT`; the server's ping interval is
    not on the wire); the name lookup has no limit of its own.
69. The record on a server (T8a, R1-5): compared whole only when it opens, a `Grew` appended;
    at most 256 frames handled a poll (`POLL_MOST`).
70. The files written whole (T8a, R1-4): beside the target, synced, renamed over; the accounts
    file 0600 at every write; a new pin merged into the pins file as it is then.
71. The mouse followed on from the last turn start (T8a, R3-1: `pointer::Follower`,
    `track::follow_on`), its equivalence with the whole re-follow a test's; an entered stack
    still clones the timeline; Auto Combat's check and the inputs' comparison still whole passes.
72. The bot's command bound (T8a, R3-4): `FOUGHT_LIMIT` counts the bot's commands since a
    human's last; a layout's `book.schools` must be the four the book draws (R4-4).
73. The harness's own checks (T8b1, R7; the runbook): `xdd.dll`'s ack has ten fields (an older
    one stops a run: `setup.sh` again for every run directory), `H3_RUN` refused without
    `WINEPREFIX`, the prefix's `d:` checked against the run directory; a run fails on the clip
    clock's field not 1, on an empty `VIDEO.VID` at a result window, and in a sound run on an
    unparsed sample or more than 32 handles; the start sound must be a new sample still playing
    at the opening, loops counted; a paused sample answers 8 (Miles not checked); `wineserver -w`
    waits at most 60 s.
74. The starting bonus Gold is not verified (T8b1, R7-8): no probe of its picture or memory
    address of the bonus is known, so `gold_bonus`'s clicks stay blind (a lost click would leave
    Artifact: a hero's numbers off, the M4 trap).
75. The result window's clips (T8b1, R2b-1, R2b-3, R2b-5): at most 64 MiB of decoded frames a
    clip (`clip::BUDGET`, about ten times `defendloop`'s); a clip that cannot be read or decoded
    leaves the slot bare (a warning); the pair decoded when the battle's end is known, never for
    a record file's window.
76. `h3-bink` (T8b1, R2a): an empty video packet refused as `Invalid` (FFmpeg refuses it too; an
    existing variant reused); a 256 × 120 sample's hashes from FFmpeg 9.0.2 (PyAV 19.0.2, its
    `bink.c` as 6.1.1's); the motion bound and a residue cut short checked equal to FFmpeg's.
77. `h3-view video` (T8b1, R2b-2, R2b-6): an archive entry whose name is not a bare file name is
    skipped and fails the run; names printed through `printable`.
