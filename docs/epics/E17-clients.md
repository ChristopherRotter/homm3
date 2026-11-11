# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12), M1 (2026-10-13), M2 (2026-10-14), M3
(2026-10-23), M4 (2026-10-27, T3c), M5 (2026-10-31, T8c), M6 (2026-11-10, T7c);
M7 (the battle client's last steps) refined at M6's closure (2026-11-10): **E17 ends with M7**
(the user, 2026-11-10); the adventure lane's harness H1 and H2 done (2026-10-19).
Each closed milestone's tasks with their dates: `E17-archive.md` (with this line's earlier
text). Refined 2026-10-12 (E17a, the asset notes, done the same day). Decisions:
[ADR-0020](../adr/0020-clients.md) (accepted at the M0 checkpoint, point 7 rewritten there).
Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`; done tasks in
full: [`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset
notes are there). Next: M7 T3 (`docs/lanes/client/QUEUE.md`). Since T9c (2026-10-23) this epic
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
- **E17 ends with M7** (2026-11-10, M6 T7c): M7 builds M6's answers and the battle as a
  hosted entry, then closes the epic. Ranked play, the pause by agreement, the server browser
  and a login dialog leave E17: E27 (the game shell: the main menu and the screens outside
  play) comes first, then a refinement of how ranked play joins the client. The first ranked
  play launches through the website (E21's portal); once HotA's content is in, its additional
  screens are weighed for reuse. Where the password is typed is decided at that refinement.

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

### Closed milestones M0 to M6

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

### M7 — the battle client's last steps

Refined at M6's closure (T7c, 2026-11-10) with the user, **shorter than first planned**: ranked
play, the pause by agreement and the server browser are not built in E17 ("Decisions", "Later").
M7 builds M6's answers and makes the battle a library entry a host's window runs (the map
client's battles, and later any front end), then closes the epic. The tasks in
`docs/lanes/client/QUEUE.md`, each cut with the user at its start when it outgrows half a
context.

- **T1, the answered slips** (cut in four with the user at its start, 2026-11-10):
  - **T1a done 2026-11-10** (the combat options dialog and the keys F6 to F8;
    [log](E17-log.md#m7-t1a--the-options-dialogs-slips-and-f6-to-f8-on-every-turn-2026-11-10)):
    CLT-Q43 (2) and (3), Q44 (c) in the client's profile; Q74's one click and Q75's F6 to F8 on
    every turn with the acting stack's reach in both.
  - **T1b done 2026-11-10** (the other answered slips;
    [log](E17-log.md#m7-t1b--the-other-answered-slips-2026-11-10)): CLT-Q70 (b) a heroless
    side's dialog and army info windows in its own player's colours, Q71 (b) a held panel
    button let go when a dialog takes its key's release (the original's as built until T1c),
    Q77 (b) the pictures box's rows; each in the client's profile.
  - **T1c done 2026-11-10** (the reads and captures;
    [log](E17-log.md#m7-t1c--the-reads-and-captures-the-answers-name-2026-11-10)): Q73's
    0x5994B0 read (the original starts a combat tune as the bar rises: the client's a copy in
    effect), the stills `stats-ability` (R2-1), `stats-held` (Q71: the pressed frame kept, the
    original's profile as built) and, with sound, `options-sound` and `options-speed-sound` (Q74:
    one click); the harness's `key-down:`/`key-up:` and sound stills.
  - **T1d done 2026-11-10** (the tools;
    [log](E17-log.md#m7-t1d--the-tools-2026-11-10)): `--close-result`; `Changes::after` and the
    cast's stop under test.
- **T2 done 2026-11-11** (the DEF button's own loop, CLT-Q72 (a), T7b's R3-4, M6 review item
  52; kept whole with the user; [log](E17-log.md#m7-t2--the-def-buttons-own-loop-2026-11-11),
  design §13.1): in both profiles, while the mouse holds an enabled DEF button (the panel's, the
  stats dialog's OK and cast button, the options dialog's, a box's) keys and the right press are
  dropped, the right release ends the hold with its click, the moves reach the button alone and
  the battle and the stats dialog's creature stand; a disabled panel button takes no press. The
  harness's `down:`; the stills `hold-right`, `stats-hold-right`, `options-hold-right`, exact.
- **T3, the battle as a hosted entry** (the map client lane's request for E24 M4 T5, the inbox of
  2026-10-31; cut in four with the user at its start, 2026-11-11, who agreed the entry's shape for
  the map client lane: the host owns the window, its loop and the frame, the battle drawn
  natively as any element of it, so that a later layout, HD's, shows it as a dialog over the
  adventure screen; design §13.2):
  - **T3a1 done 2026-11-11** ([log](E17-log.md#m7-t3a1-and-t3a2--the-battle-clients-library-and-the-hosted-element-2026-11-11)):
    `h3-view`'s binary became the library crate `h3-battle-client` (the user: an adapter crate,
    left out of the coverage gate as the binaries are), the `h3-view` command line its binary,
    `h3-view` the pure library alone; nothing changed in behaviour.
  - **T3a2 done 2026-11-11** (same log entry): the window's loop as the element
    `window::Hosted` (with `Prepared` and `Setup`), `h3-view window` its first host. Built
    without the original data (the user's choice); checked by the next session with the data
    staged (the windowless host test, `window --check`, `drawn`, the gate, `check.sh`).
  - **T3b**, cut in two with the user (2026-11-11): **T3b1** a whole game's record (format 4)
    gives the battle the element plays and hands back with its outcome (the game's battle
    messages as the battle's record; `h3-view window` on a game record; the battlefield
    picture's variant from the wire, protocol v35's `Observation.background`, CLT-Q4; a
    windowless test host; the map client lane's inbox told the API): what E24 M4 T5 needs, which
    shows golden replays' battles. **T3b2** the same battle played live through the host's game
    session (the element giving its commands to the host), before T3c.
  - **T3c:** the sound on the host's device and the music through the common `h3_ui::music` (the
    end's tune after the combat music's 900 ms fade; the focus lost and regained: the map client
    lane's notes of 2026-10-27 and 2026-10-30); the frame widened by
    `h3_render::screen::Widening` if measured faster (2026-10-28).
- **T4, M7's and E17's closure:** a fresh reviewer, the captures where the battle screen changed,
  the review list with the user; the epic's open threads placed ("Later": E27, E24 or the
  backlog); E17 closed (the backlog's row by a `shared/` pull request).
- **Exit:** M6's answers built and recorded; a battle runs inside a host's window to its result
  and hands back; every open thread of E17 placed in another epic or the backlog.

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

### Later (placed at M7's closure)

**Not in E17** (the user, 2026-11-10; "Decisions"): **ranked play** (the ladders, the queue
and the accept step, the ratings, the leaderboards), the pause by agreement, the server browser
and a login dialog in the window: after E27, by a refinement of their own; first through the
website (E21). The server lane's notes on what a live client must do stay in `NOTES.md`, "The
live client". Placed at M7's closure (T4): HD layouts, graphics options beyond `faithful`, the
client's options (volumes; a two-hex stack's step one hex back by the mouse as a gameplay
option, off by default: CLT-Q25), video. The adventure map and the full game viewer: E24, the
map-client lane. When the battle client runs inside an adventure game (M7 T3's entry, E24 M4
T5): **M6 T5e**, the captured artifacts' message after Eagle Eye's (deferred by the user, 2026-11-07;
`captured_artifacts_message` 0x4772B0, design §12.21: GENRLTXT 31, eight artifact pictures a
box through `h3_view::message::pictures`, a pickup sound each), its list the adventure's capture
(HER-ART-004) told to the winner's owner; its reading of 0x4772B0 and of the artifact picture
(type 8) to finish then, a server's seat with it.

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
2026-10-31, M6 2026-11-10); in the log,
[`E17-log.md`](E17-log.md#accepted-review-lists-m1-to-m6) (M1 and M2 moved there on 2026-10-20,
M3 at its review, T9b, M4 at its review, T3c, M5 at its review, T8c, M6 at its review, T7c).
The next milestone's list starts here.

**M7** (for its closure, T4):

1. F6 to F8 on every turn (T1a, CLT-Q75 (a), *provisional* where marked; design §12.16): taken
   once the battle's last action played (`Battle::takes_keys`); a key during an action dropped
   as the original's queue is emptied at the next turn's start (*provisional*: the original's
   keeps it for a remote turn); the acting stack's reach on a turn no human plays drawn only
   for a stack whose commands the client knows (*provisional*: none on a seat's opponent's
   turn, the server sending no commands for it). Not recorded.
2. The other slips (T1b, CLT-Q70 (b), Q71 (b), Q77 (b), *provisional* where marked): a
   heroless side's colours its own player's where it has one (a colour, or a human's: CLT-Q6's
   red or blue), the viewing player's for a side with none (*provisional*: neutral creatures, no
   player to take); a held button let go on a release the stats dialog, a message box or popup,
   the combat options, the spell book or an aim takes (*provisional*: the book and the aims
   beyond the answer's dialogs and popups); the pictures box's size unchanged by its rows'
   fix. Not recorded (the stills and sequences the original's).
3. The reads, captures and tools (T1c, T1d; *provisional* where marked): the harness's sound
   stills and `samples` (the set's runner, `set` only: a sequence's sounds still unrecorded) and
   `key-down:`/`key-up:` (in a still only a panel button's hotkey held into a dialog; a release
   over the battle refused); the effects' bars of a sound still recorded as differing (578
   pixels) rather than drawn at the run's volume; `--close-result` pressing Eagle Eye's boxes
   after the result window too.
4. The DEF button's own loop (T2, CLT-Q72 (a), design §13.1; *provisional* where marked): in
   both profiles; nothing posted at the release (the battle and the stats dialog take the mouse
   at its next move: the map's recorded release gives the point's hint); the boxes' and the
   result window's buttons ended by the right release too (no still); a disabled panel button
   takes no press (before, held unpressed); the viewer's own keys still act during the hold; a
   hotkey's button let go by the mouse's press in both profiles (the original's keeps it drawn
   pressed); the panel's press samples not built. Recorded: `hold-right`, `stats-hold-right`,
   `options-hold-right` (exact, the battle's and the creature's standing found by them).
5. The battle's library and the hosted element (T3a1, T3a2; design §13.2, *provisional* where
   marked): the crate `h3-battle-client` (the user's choice, not reviewed again) with the
   binary's modules all public and the `h3-view` binary in it; the element's shape (`Prepared`,
   `Setup`, `Presses`, `Hosted` and its pass: `timeout`, `before`, `event`, `closed`, `after`,
   the cursor, `sound`, `due`/`showed`; `Flow`, `Took`), *provisional*: the host's tick source
   handed to each call (`&dyn Fn() -> u64`), the platform's `Event` and cursor image in its API,
   the window's own keys split (the battle's clock and sound its own, fullscreen, vsync and the
   cursor the host's), its sound still on a device of its own until T3c. Not recorded (nothing
   the original draws changed); checked with the data staged by the next session.
