# E24 — The adventure map client

Status: **refined** (2026-10-23, T0, with the user); **M1 closed** (2026-10-24, T6b: the
reviewers, the review list accepted by the user); **M2 refined** (2026-10-26, M2 T0, with the
user); **M2 closed** (2026-10-28, T6b2c: the user's hand test, the review list accepted at
T6b2a); **M3 refined** (2026-10-28, M3 T0, with the user); **M3 closed** (2026-10-31, T7c2: the
review list accepted at T7c1); **M4 refined** (2026-10-31, M4 T0, with the user); **M4 closed** (2026-11-23,
T6: six reviewers, their findings fixed or asked; the review list accepted at M5 T0); **M5
refined** (2026-11-24, M5 T0, with the user: the milestones split, M5 the hot seat on the
adventure map, M6 to M8 sketched); next M5 T2 (`docs/lanes/map-client/QUEUE.md`).
Lane: map-client (`docs/lanes/map-client/HANDOFF.md`: the autonomy contract, the user's asset
notes). Opened 2026-10-23 when the user split the client lane at E17 M3's closure (CLT-Q18).
Decisions so far live here (no ADR of its own yet; ADR-0020 and ADR-0024 bind it as they bind
E17). Working detail (code readings, formats, layouts): `docs/epics/E24-design.md`, created by
the first task that has detail to record; done tasks in full: [`E24-log.md`](E24-log.md).

## Goal

The adventure map drawn with the original's own assets, truthful to the original down to the
pixel and the frame as E17's battle is: stills of a map first, then the map moving in a window,
the adventure screen around it, the engine's whole games shown, and whole games played. On E17's
foundation (the decoders `h3-assets`, the compositor `h3-render`, the SDL adapter `h3-sdl`, the
string catalogue) and its capture harness, all common paths of the two client lanes. Battles
inside a game are the battle client's (E17); this epic hands them over.

## Decisions (the user, 2026-10-23, T0; E17's binding here too)

- **E17's decisions hold** (`docs/epics/E17-clients.md`, "Decisions"): exhaustive, automated
  diffs against the original's captures (the harness under Wine, never the installation); the
  original's drawing bugs fixed and reproduced only for the comparisons (`h3_render::Bugs`); its
  redraw artifacts not copied; graphics options per effect, `faithful` the default and the only
  setting the comparisons check; asset tables per expansion, committed, identifiers and numbers
  only; loading at checkpoints, as fast as the original; every text through the string
  catalogue; **every oddity of the original an open question** (`MCL-Q<n>`, the lane HANDOFF's
  autonomy contract).
- **M1 is the map view and the screen's frame** (T0): the panels right of and below the map
  (minimap, hero and town lists, status window, resource bar, buttons) wait for M3, when the
  pieces of `h3-view` they need sit in a common crate.
- **The map moving comes before the panels** (T0): M2 the animated map in a window, M3 the
  adventure screen.
- **Test maps through `h3_map::writer`, extended by this lane** (T0): what the situations need
  and the writer lacks (rivers, a tile's pictures and flags as given, any object template) this
  lane adds to the adventure lane's writer in **additive `cross-lane` pull requests** (the
  user's consent for that kind of change, 2026-10-23), each noted in the adventure lane's inbox;
  nothing the writer writes today changes.
- **The string catalogue stays common** (T0; `clients/text/`, as T9c made it): both client
  lanes add keys.
- **A text the original sets off its centre is centred** (the user, 2026-10-31, CLT-Q28 at E17
  M5 T8c, for both client lanes: `docs/epics/E17-clients.md`, "Decisions"): in the client's
  profile the text is centred, the original's place kept for the comparisons only
  (`h3_render::Bugs`, as the cursor's x); a reading or a recording that finds such a slip fixes it
  so, with no question needed (recorded here at M3 T7c1; none found in the map client yet).

## How the viewer sees a map (T0's finding; the types since T2: `h3_map_view::art`, `live`, *provisional*)

- **Two inputs.** The engine's import (`h3_map::import`) draws nothing and keeps no pictures: no
  terrain picture, no mirror flags, no template's DEF; it imports 5 of the 160 shipped maps
  whole, and its `ObjectView`s carry no index into the map's objects. The raw map
  (`h3_map::parse` → `RawMap`) keeps all of it: each tile's terrain, river and road with their
  pictures and flags, each template's DEF, masks and priority, each object's anchor. So the
  viewer takes the **static art** from the raw `.h3m` and the **live part** (which objects stand,
  what a random object became, the owners, the heroes, what the viewer has explored, the
  moment) from a second input. In M1 that input comes from the original's memory for the
  comparisons (`h3mem.py`, an additive harness change) or from the map's start where it needs
  no draw; the engine fills it from M4 on.
- **Random objects.** The original draws a shipped map's random objects (monsters, resources,
  artifacts, towns, dwellings) when the game starts, and the harness saw them differ from run to
  run (E17 design §6.3: the piles' amounts): a comparison on such a map reads what the original
  drew from its memory, never assumes the engine's draw (T4b: each object's template in memory,
  the copy a new game made for a random one, `E24-design.md` §2.7).
- **Where the reading starts** (`docs/exe/map.toml`): each map tile keeps a draw list of the
  objects over it; `place_object_tiles` (0x505B20) adds an entry per tile of an object's
  footprint through 0x505230, `object_tile_layers` (0x505060) gives each tile a draw layer from
  the template's masks (a flat template, the special grounds, keeps layer 0), and the objects'
  shadow pass is the encoding 3 drawer 0x47D930 (E17 design §3.3). The view (T1a's reading,
  `E24-design.md` §1): eleven passes over the whole view, tile (x, y) at (32 (x − vx),
  8 + 32 (y − vy)) clipped to (8, 8)–(600, 552), the view's top-left at adventure manager +0xE4.
  The objects (T1b's reading, `E24-design.md` §2): a 32 × 32 piece per tile of each object's
  draw list, the list ordered by its cells' layers; flat objects first, then every shadow, then
  the bodies by layer with heroes and boats as two-row objects (after layers 1 and 2), then the
  path's arrows.

## Milestones

Each milestone ends at a checkpoint with the user, after a fresh reviewer agent checked it. The
autonomy contract (lane HANDOFF) says what the agent decides alone in between. Tasks: the lane's
`QUEUE.md`, each about half a context.

### M1 — stills of the map view (closed 2026-10-24)

Tasks T0 to T6b and the exit record: `E24-archive.md` (verbatim); the build record: `E24-log.md`; the design: `E24-design.md` §1, §2. The set of 40 captured situations (`clients/map/situations/`) is exact; a change to the viewer runs `h3map.py drawn` first (the runbook).

### M2 — the map moving (closed 2026-10-28)

Scope, decisions, tasks T1a1 to T6b2c and the exit record: `E24-archive.md` (verbatim); the build
record: `E24-log.md`; the design: `E24-design.md` §3. Its decisions that bind later milestones
(in the archive): the walk's path always the engine's (M4, M5), never a client pathfinder; M3
T0's common crate for both lanes, the key table moved into it; the edge scroll's 16 pixels
(MCL-Q6). The 13 recorded sequences (`clients/map/sequences.toml`, `timelines/`) are exact; a
change to the viewer runs `h3map.py drawn` first (it checks every timeline too).

### M3 — the adventure screen (closed 2026-10-31)

Scope, decisions, tasks T0 to T7c2 and the exit record: `E24-archive.md` (verbatim); the build
record: `E24-log.md`; the design: `E24-design.md` §4. Its decisions that bind later milestones
(in the archive): `h3-ui` the one crate both viewers share (the key table by context, both
windows' own keys the map's); the panels' live part `h3_map_view::live`'s, from the original's
memory in a recording and from the engine from M4; only the clicks that change the view work,
every other button showing its press (its dialog or the engine's action: M4, M5); the cursor
over the map from given inputs (the tile's contents, the path's reach), never computed by the
client; MCL-Q22's, Q24's and Q28's fixes the client's, the original's behind `--bugs original`
(MCL-Q14, Q15 and Q21's follow-ups: M4's). The 50 situations and the 35 sequences are exact
on the whole screen; a change to the viewer runs `h3map.py drawn` first.

### M4 — the engine's games shown (closed 2026-11-23)

Scope, decisions, tasks T0 to T6 and the exit record: `E24-archive.md` (verbatim); the build
record: `E24-log.md`; the design: `E24-design.md` §5. Its decisions that bind later milestones
(in the archive): the input through `h3-protocol`'s whole-game record (ADR-0020 point 8), never
the engine read directly; one player's view; a fact the record lacks an `MCL-Q<n>` with a note
in the engine lanes' inboxes, never a rule copied into the client; the battles hosted in the
map's window through `h3-battle-client` (the binary's dependency), the host drawing the
switch's fades. The 56 situations and every sequence and scripted game are as recorded
(`drawn`); the golden replays' records play to their end headless, and in the window with
their battles. Its review list was accepted at M5 T0 (the archive's end).

### M5 — hot seat on the adventure map (refined 2026-11-24, M5 T0, with the user)

A whole game played on one machine by its human players in turn, started from the shell's
Begin: the engine's game hosted in-process by `h3-protocol`'s game session, each human seeing
its own view, every command the engine's to judge; the hand-over between humans, the turn's
clock and the battles as the original's.

- **Scope:** the player's program (the shell, then play, then the battles, in one window); the
  session in-process (a seat per human, the setup and the names by colour from `begin::Game`,
  the turn's duration on the session's clock, each human's view folded from its own frames and
  the map drawn from the current human's, each seat's record kept); live input through the
  engine (its reach and paths for the cursor, a click's path and the direction keys, lifting
  M2's input without the engine; End Turn and Move Reminder; every box M4 shows answered by the
  player as the command); the hand-over (the covered view and GENRLTXT 14, design §5.1) and the
  clock's boxes (GENRLTXT 202, 627 to 630); the battles in play (attacking, defending:
  `MCL-STUB-3`; two humans: `CLT-STUB-5`; Auto Combat: `CLT-STUB-4`); M4's carry-overs.
  **Not in M5:** the hero's screens and the visit windows (M6), the town screen (M7), live play
  against the server (M8); a computer player's turn (E31; its colours' turns end at once); the
  game's end (E10's increment 1); Load, until the engine changes a loaded game's humans (asked of
  the adventure lane by the client lane, 2026-11-21); E27 M5's system menu in play, Options and
  Save from play (the client lane's, refined once M5 hosts play).
- **Decisions (the user, M5 T0, 2026-11-24):**
  - **The milestones split:** M5 the hot seat on the adventure map, M6 the hero's screens and
    the visit windows, M7 the town screen, M8 live play against the server (the one M5 sketched
    before was all four).
  - **The engine and the server the only authority for a client** (the user's answer to
    MCL-Q54): reading a game's facts from files at run time is phased out. **Facts first:** in
    play every fact comes from the engine's view and events: no rules table of the client's, no
    map text read from the file; a fact the view lacks is an `MCL-Q<n>` and a note in the engine
    lanes' inboxes, built plainly meanwhile. **The pictures later:** the map file stays the
    source of the map's pictures (its hash checked, ADR-0020 point 8) until an ADR moves them to
    the server too, because a map the server generates should reveal its contents only as the
    player discovers them ("Later"). Recordings compared against the original keep their files
    (test tooling).
  - **The shell on top** (the user: the shell calls the map unless architecture strongly opposes
    it; least effort is no such reason). Cargo forbids package cycles and `h3-map-view`'s library
    is pure, so: the map's window, painter, sound and battle hosting move from `h3-map-view`'s
    binary into a new adapters library, **`h3-map-client`** (this lane's, as `h3-battle-client`
    was made from `h3-view`'s binary); the client lane moves the shell out of `h3-battle-client`
    into a crate of its own on top, whose binary is the player's program; the shell calls
    `h3-map-client`'s play, which hosts the battles through `h3-battle-client`, the battle again.
    Rejected: the map viewer hosting the shell; the shell calling play through a trait of its
    own with the program's `main` in the map's package (the shell's crate below the map's).
  - **A computer colour's turn ends at once** until E31's computer player gives it turns
    (`MCL-STUB-5` planned, lifted by E31's increment 2); any map plays, its computers idle.
  - **The exit measured twice:** the recorded parts (the hand-over, the clock's boxes, the
    battles' switches) as recorded under Wine; a whole two-human game on a shipped map from
    Begin to a day limit, headless by script and in the window by hand, every command the
    engine's.
  - **From M4's review list** (accepted at T0, six items asked one by one): the original's random
    picks (a computer turn's music, the pickup sounds, an unnamed town's name) stay in turn for
    recordings and records, and play draws them from the client's own seeded generator (a
    town's name the engine's where it names one); one record reader shared with the client lane
    (the place agreed with it at T2); the hero's pose at the attack and the panel's buttons after
    a retreat fixed (T1a, done); the original's display of a computer's turn read where play
    shows one (after E31: "Later").
  - **From M4's questions** (MCL-Q50 to Q56, answered one by one at T0): MCL-Q51 recorded, then
    read for the draw (T1b); MCL-Q52 read, (b) meanwhile (T1c); MCL-Q53, Q54 and Q55 asked of the
    engine lanes (their inboxes), the built forms meanwhile; MCL-Q50 and Q56 kept as built.
- **Recorded and compared as M4:** the hand-over and the clock's boxes recorded under Wine in
  hot seat (the harness's `hot_seat`) and compared frame by frame; play's frames where a scripted
  game gives both the same commands with no random draw between them.
- **Tasks** (the queue has each in full): T0, M5 refined (**done** 2026-11-24: the log's "M5
  T0"); T1, M4's hand test and carry-overs, cut in three at its start: T1a the hero on its tile
  through a battle and the panel after a retreat (**done** 2026-11-24: the log's "M5 T1a",
  design §5.22), T1b MCL-Q51 (**done** 2026-11-25: the log's "M5 T1b", design §5.23; the
  hand test's notes, still blank, wait for T2), T1c MCL-Q52 (**done** 2026-11-25: the log's "M5
  T1c", design §5.24); T2, the layering (no behaviour changed), cut in three by the user at its
  start: T2a the binary's adapters moved into `h3-map-client` (**done** 2026-11-25: the log's "M5
  T2a"), T2b play's entry for the shell and the client lane's move queued (**done** 2026-11-25:
  the log's "M5 T2b"; the API the user agreed for the client lane), T2c one record reader shared
  with the client lane (**done** 2026-11-25: the log's "M5 T2c"; `h3_protocol::record`, MCL-Q57);
  T3, the session in-process; T4, live input through the engine; T5, the hand-over and the clock; T6, the battles in play; T7, a whole game; T8, M5's
  closure. A task that grows is cut at a seam agreed with the user.
- **Exit:** the hand-over, the clock's boxes and the battles' switches as recorded; a two-human
  hot-seat game on a shipped map played from the shell's Begin to a day limit, headless by
  script and in the window by hand, every command the engine's; no rules table or map text read
  in play; M5's review list accepted; a hand test on the user's machine.

### Later (refined when they come)

- **M6, the hero's screens:** the hero screen, the meeting, the spell book with the adventure
  casts (E08 increment 15), the army dialogs, the visit windows (a hire, a Shipyard, a Market,
  an Altar with its level-ups drawn over it, a Hill Fort); the inbox's notes for them.
- **M7, the town screen:** the town lane's builds (`TownCommand::Build`, `TownOffer`), the
  recruitment, the tavern, the Mage Guild and its book, as play needs them.
- **M8, live play against the server.**
- **The pictures from the server** (M5 T0's decision): an ADR amending ADR-0020 point 8, then the
  cross-lane work (the import keeping the map's pictures, the wire revealing them as discovered);
  placed with the user after M5.
- **A computer's turn as the original shows it** (M4's review item 6, accepted at M5 T0):
  `ai_hero_turn` and its callers read once E31 gives the computers turns.
- A spectator's view of a game (M4 T0); a loaded saved game's draw lists, if a comparison needs
  one (M1 T6b); shipped maps with towns once E09 builds them.
- HD layouts, graphics options beyond `faithful`, video.

## Research items from the asset notes

| Item | Where |
|---|---|
| 9. The adventure map's object draw order (the user: neither they nor VCMI got it exactly right); the objects' shadows | **Closed** (T4b): M1 T1b (read, `E24-design.md` §2), T4a (its frames, §2.6), T4b (random objects, shipped maps' views, §2.7); heroes and boats among the layers since T5a (§2.8) |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | later |
| 7. The HD patch as a source of fixes and for HD | later |

## Risks and known gaps

- **The original's moment.** A still is exact only at a known game time: whether the harness's
  virtual clock (`xdd.dll`) drives the adventure map's animations and colour cycles as it drives
  the battle's (T3b2: it does); the palette step is read from memory, the menus run in real time.
- **Random objects drawn in real time** (above): the comparisons depend on reading them from the
  original's memory.
- **The engine lags the viewer:** a record lacks facts the original shows (M4's MCL-Q36 to Q55:
  a hero coming into sight, a mine's production, a Box's untold rewards, the days without a
  town), each built plainly meanwhile and asked of the engine lanes; the towns' screens (E09) and
  the end of a game (E10) come with their epics.
- **The engine's games against the original** (M4): only what no random draw separates compares
  frame by frame (scripted commands on maps without random objects between them); a computer
  player's moves cannot be scripted in the original, so its turn is compared by what the viewer
  sees of it, not by its choices.
- **M5 rests on other lanes:** the shell's own crate (the client lane), a loaded game's humans
  (the adventure lane), the facts the view lacks (the adventure and server lanes); T2 builds
  `h3-map-client` first so the shell's move has a target, and each gap is built plainly
  meanwhile.
- **The writer belongs to another lane:** each addition is a `cross-lane` pull request; a
  situation waits for its merge.
- **Wine is not Windows** (E17's risk): a difference only Wine could cause needs a check on
  Windows.
- **The map's input without the engine** (M2): a direction key steps the hero through obstacles
  (the original's `hero_step_allowed` is the engine's rules, M5), a click shows only a path a
  recording or a situation gave (the engine's paths: M4, M5), and no step costs points (the
  walk on after a held key's release has no end but the queue's: MCL-Q10).

## Review lists

M1's review lists (three reviewers, T6b; the list accepted), M2's (accepted at T6b2a), M3's
(six reviewers at T7, the list accepted at T7c1) and M4's (six reviewers at T6, the list
accepted at M5 T0 with the user's six answers): `E24-archive.md`. Later milestones add theirs
here.

**M5's** (queued for its closure, T8):

- `h3-map-client` left out of the client workspace's coverage gate as `h3-battle-client` is
  (`clients/scripts/check.sh`; M5 T2a): its code was `h3-map-view`'s binary, under `/src/bin/`,
  out of the gate already; its own tests, `window --check`, `drawn` and the records exercise it.
- Play's entry until T3 (M5 T2b, *provisional*): the game started in-process and its first
  human's start shown standing with M2's input (the user's choice at T2b); the window's quit key
  ends play with `End::MainMenu`, the window closed with `End::Quit`, until E27 M5's system menu
  gives the four ends; play's report (the window's lines) on standard error.
- The record reader in `h3-protocol` (M5 T2c, MCL-Q57: the user's place, beside the writer):
  it reads the lines alone (the header's checks, the numbered messages, the refusals), each
  client keeping its own record type and readings; the line's bound 64 MB for both kinds (the
  live client's message bound; `h3-view` had 16 MB), *provisional*.
