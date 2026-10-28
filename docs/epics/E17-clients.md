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
      `result-surrender` exact, `end`'s last frame as recorded.
    - **T4b2, the clip:** its slot's Bink clips (a port of FFmpeg's decoder, the user's choice,
      CLT-Q29), the harness given them.
  - **T4c, Auto Combat and the tactics phase:** the panel dimmed, the tactics phase's panel
    (0x46C050).
- **T5, what a played battle shows:** the created stacks (`StackCreated`: Summon Elemental,
  Clone; a Pit Lord's `RoseFromCorpse`) with their dissolve (`screen_dissolve` 0x602DC0's blend,
  Sacrifice's victim's too), a clone's `Vanished`, Teleport's dissolve; the panel's side by each
  turn's controller (R3-1: Hypnotize); recorded.
- **T6, a game on a server:** joined over the server's WebSocket by a seat token: the hello and
  the welcome's checks, the key pinned per address, the content check (`docs/lanes/client/NOTES.md`,
  "The live client"); a record that opens mid-battle (R2-8, with M4 T3a's R1-3) for a join and
  a re-join after a drop or a server restart. Its dependency (a WebSocket client with TLS): the
  user's decision, at the task.
- **T7, accounts:** the account's key from the login and the password (argon2id, Ed25519; their
  dependencies the user's decision, at the task), the login with the welcome's check, seats by
  account, `Rekey` after a password change, the new goodbyes.
- **T8, M5's closure:** a fresh reviewer, the recipes' starting bonus set to Gold (M4's item 15),
  every capture afresh, the review list, the next milestone refined.
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
