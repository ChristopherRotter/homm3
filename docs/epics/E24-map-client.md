# E24 — The adventure map client

Status: **refined** (2026-10-23, T0, with the user); **M1 closed** (2026-10-24, T6b: the
reviewers, the review list accepted by the user); **M2 refined** (2026-10-26, M2 T0, with the
user); **M2 closed** (2026-10-28, T6b2c: the user's hand test, the review list accepted at
T6b2a); **M3 refined** (2026-10-28, M3 T0, with the user); **M3 closed** (2026-10-31, T7c2: the
review list accepted at T7c1); **M4 refined** (2026-10-31, M4 T0, with the user); next M4 T1
(`docs/lanes/map-client/QUEUE.md`).
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

### M4 — the engine's games shown (refined 2026-10-31, M4 T0, with the user)

A whole game the engine played, shown in the window as the original shows it to one player: the
original's own reactions to a game's events read and recorded first (its memory the live part,
as in M1 to M3), then the engine's records as the input, through `h3-protocol` (ADR-0020 point
8); its battles handed to the battle client's viewer in the same window.

- **Scope:** **Part A, the original's reactions, no engine needed:** the turn's change (T1: End
  Turn's action and its question when a hero can still move; the turn's end and the next
  player's start; the new day's, week's and month's steps and messages; another player's turn,
  the status window's kind 5, `AdStatNx.pcx`, the hourglass, 0x453250, with its tile, hint and
  cursors, 0x40DEB0; a computer player's moves where the viewer sees them); a visit's results
  (T2: the adventure's message box, its text, pictures, buttons and sounds, for the finds and
  questions the engine's visits make, the level-up's choice among them; a resource found, kind
  6, 0x451220, 5 seconds; a text, kind 7, 0x451820, design §4.8; an object taken, the map, the
  minimap and the panels redrawn; a flag raised; a boat boarded and left; a spell cast on the map
  recorded, MCL-Q14; a monster beaten moved to T5 by the user at T2d's start). **Part B, the engine's games through the protocol's
  whole-game record:** a record's start (T3: the static art from the map it names, the live part
  from its first observation: each object by its identity, a random object's type, the owners,
  the heroes, the boats, the explored tiles, the kingdom, a hero's status, the hover's facts; a
  town's own name from the map); a record played over time (T4: each event as Part A built it,
  the walks along the engine's paths as M2's, the record's answers in its boxes, the viewer's
  controls). **Part C:** the battles handed over and back (T5; a monster beaten, the map after a
  battle won, with them). The closure (T6). **Not in M4:**
  the hero, town, meeting and kingdom-overview screens, the quest log and every dialog but the
  message boxes (M5); play (M5); a spectator's view; shipped maps with towns (E09: of the 160,
  the engine plays 8 whole, 128 stop at a hero generated at a main town; M4's games are on those
  8 and the writer's maps); a loaded saved game's draw lists (the objects gone before the save:
  not read, M1 T6b).
- **Decisions (the user, M4 T0, 2026-10-31; every recommendation taken):**
  - **The input through the protocol, the server lane first** (ADR-0020 point 8 kept): the
    viewer plays `h3-protocol`'s whole-game messages, and an engine replay becomes a record by
    running the protocol's session in-process, as `h3-view render` does for a battle. The
    protocol has none yet (it does not depend on `h3-game`): the server lane builds the whole
    game's wire messages, a session hosting one game in-process and its per-viewer record first,
    as the start of E16 M5, refined with the user there (its inbox lists this lane's needs);
    Part A, which needs no engine, runs meanwhile. Rejected: the engine read directly with
    ADR-0020 amended (two input paths until the protocol's), this lane writing the protocol's
    adventure part (the server lane's crate).
  - **One player's view:** a record shows the game as the original shows it to that record's
    player: its fog, its own turns as played (the walks along the engine's paths, its message
    boxes with the answers the record holds), the other players' turns as the original shows a
    computer player's (the status window's kind 5, their moves where the player sees them). A
    spectator's view later (`h3_game::Viewer`'s spectator, the server lane's).
  - **The message boxes only:** the adventure's message boxes and questions (a find, a guard's
    question, the week's and month's messages, a level-up's choice), each with the answer the
    record holds, as the original shows them to that player; the hero's, a town's, a meeting's
    and the kingdom overview's screens stay M5's.
  - **The battles in the same window:** the battle client's viewer draws a battle of the game in
    the map viewer's window, as the original switches screens, and hands back with the result
    applied on the map. The client lane turns its battle window into a library entry the map
    viewer hosts (its inbox); built late in M4 (T5).
  - **From M3's answers:** MCL-Q15 (b), since the engine's treasury reaches 2³¹ − 1
    (`h3_adventure::MAX_GAIN`), past the bar's seven digits: the client writes a wider amount so
    that it fits, its form a review item, the original's behind `--bugs original` (T3); MCL-Q21
    (b): a map whose hints would name nothing is refused at the engine's import (the adventure
    lane's inbox has the cases) and at the viewer's load (T3); MCL-Q14: a cast recorded in the
    original first (T2), the fix if the stale bar shows.
- **Recorded and compared as M1 to M3:** Part A's new situations and sequences exact on the whole
  screen (`drawn` with them); Part B: the engine's start drawn as the original's on the maps both
  play, and scripted games (the same commands, no random draw between them) played in the engine
  and in the original compared frame by frame, the original's visual moment (the objects'
  phases, the clock) taken from its recording.
- **Tasks** (the queue has each in full): T0, M4 refined (**done** 2026-10-31); T1, the turn's
  change, cut in two by the user at its start (2026-11-16): T1a, the turn's end and the next day
  (End Turn and its question, the new day, week and month and their messages: the first message
  box; **done** 2026-11-17, the log's "M4 T1a"), T1b, a computer player's moves in sight with
  the status window's kind 5, cut in two by the user at its start (2026-11-17): T1b1, the reading
  and the recordings (**done** 2026-11-17, the log's "M4 T1b1"), T1b2, the viewer playing them
  (**done** 2026-11-17, whole: the user kept it uncut; the log's "M4 T1b2");
  T2, a visit's results, cut in four by the user at its start (2026-11-17, by dialog family):
  T2a, the finds told (the box with pictures, the status window's kinds 6 and 7, the turn's two
  boxes T1a left), cut again at its start (the agreed fallback): T2a1, the reading and the
  recordings (**done** 2026-11-17, the log's "M4 T2a1"), T2a2, the viewer (**done** 2026-11-18,
  whole: the user kept it uncut; the log's "M4 T2a2"); T2b, the questions and
  choices, cut in two by the user at its start (2026-11-18, as T2a): T2b1, the reading and the
  recordings (**done** 2026-11-18, the log's "M4 T2b1"), T2b2, the viewer (**done** 2026-11-18,
  whole: the user kept it uncut; the log's "M4 T2b2"); T2c, a level-up's choice (**done**
  2026-11-18, whole: the user kept it uncut; the log's "M4 T2c"), T2d, the map changed by a
  visit, cut in three by the user at its start (2026-11-18, by family; "a monster beaten" moved to
  T5): T2d1, a flag raised (**done** 2026-11-18, the log's "M4 T2d1"), T2d2, a boat boarded and
  left (**done** 2026-11-19, whole; the log's "M4 T2d2"; MCL-Q35), T2d3, a spell cast on the map
  (**done** 2026-11-19, whole: Summon Boat, the user's choice; the log's "M4 T2d3"; MCL-Q14 seen);
  T3, a record's start (after the server lane's whole-game record
  and the adventure lane's object identity); T4, a record played; T5, the battles handed over
  (after the client lane's entry); T6, M4's closure. A task that grows is cut at a seam agreed
  with the user.
- **Exit:** Part A's situations and sequences exact on the whole screen (or each difference an
  `MCL-Q<n>`); the engine's starts and scripted games equal to the original's frames; a golden
  adventure replay, as the protocol's record, viewable from start to end in the window with its
  message boxes and its battles; M4's review list accepted; a hand test on the user's machine
  (the map's sound by ear on a whole map, the user at M2 T4b2: the transitions sounded unclean;
  MCL-Q11's `HORSE20` to `HORSE29` listened to for its (c)).

### Later (refined when they come)

- **M5, hot seat:** whole games played in-process: the handover screen when the player who must
  act changes, the level-up dialog (the inbox), the hero and town screens as play needs them;
  then live play against the server.
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
- **The engine lags the viewer:** 8 of 160 shipped maps import whole (128 wait for towns, E09),
  the observation has no object identity, and the protocol has no whole game: M4's Part B waits
  for the last two (the adventure and server lanes' inboxes), Part A needs none.
- **The engine's games against the original** (M4): only what no random draw separates compares
  frame by frame (scripted commands on maps without random objects between them); a computer
  player's moves cannot be scripted in the original, so its turn is compared by what the viewer
  sees of it, not by its choices.
- **The writer belongs to another lane:** each addition is a `cross-lane` pull request; a
  situation waits for its merge.
- **Wine is not Windows** (E17's risk): a difference only Wine could cause needs a check on
  Windows.
- **The map's input without the engine** (M2): a direction key steps the hero through obstacles
  (the original's `hero_step_allowed` is the engine's rules, M5), a click shows only a path a
  recording or a situation gave (the engine's paths: M4, M5), and no step costs points (the
  walk on after a held key's release has no end but the queue's: MCL-Q10).

## Review lists

M1's review lists (three reviewers, T6b; the list accepted), M2's (accepted at T6b2a) and M3's
(six reviewers at T7, the list accepted at T7c1): `E24-archive.md`. Later milestones add theirs
here.

### M4's review list

The M4 review list starts here (the autonomy contract's "queue for the milestone review").

- **M4 T1b2, speed 4's single draws on the window's clock** (*provisional*): a computer's steps
  at Computer Walk Speed 4 take no time on the map's clock (the original draws each once with no
  wait, as fast as it can); the window shows each for 20 ms of its own clock (`HOLD_MS` in the
  binary's `window.rs`), the map's clock held at their millisecond meanwhile, then the
  millisecond's own change. A recording gives no steps inside End Turn's millisecond, so only
  the engine's records (Part B) or a hand-made sequence show it (the T1b2 check used `ai-walk`'s
  walk at speed 4 in a scratch map folder).
- **M4 T1b2, a computer's turn from a recording** (*provisional*): its walk is the recorded
  timeline's `hero` lines in its turn (one hero; several, or a hero of another level, refused);
  the turn tables' standing heroes matched to the map's by owner and class (two alike refused, a
  boat's move refused); its sounds and music (`AITheme0`–`2`, the steps' sounds) not played;
  input during it dropped, the mouse's point taken; kind 5's step at every step's start, its
  50 ms deciding (which call steps it is not read); the busy cursor's hot spot the default
  cursor's (never drawn: hidden from the first seen step); the cursor's type set back to the
  adventure arrow's at the turn's end; the redraws missed caught up there before the next day's
  whole redraw (recorded, not read).

- **M4 T2a2, a find's facts without the engine** (*provisional*; design §5.3, *Built*): a
  sequence's finds take their facts from the map and the live part (the Windmill's, the Witch
  Hut's and a Shrine's from the entrance words, the Stone's, the Axis's and the Temple's visits
  from the marks, a known skill or spell from the contents, else not known); the Learning Stone's
  1 000 from the tables (no Learning known); a Spell Book and Wisdom taken as present; the days
  without a town −1 at 0; ArrayTxt 129 at one day left (not read); the selection's window built
  again at the days left's close (recorded, not read); the crest's caption's first letter
  upper-cased (PlColors' "red" shows "Red"; the loader not read). M4's Part B (T3, T4)
  takes the facts from the engine's events instead.
- **M4 T2a2, an object taken** (*provisional*): the pickup's sound `pickup01` to `pickup07` in
  turn (the viewer's own draw, not the game's roll at 0x4A4D18; no frame shows it); a step onto a
  pile or an artifact as recorded (the hero beside it, its points spent; a key's step only, a
  path's walk onto one not built); an artifact picked up taken as worn, its primary skills from
  the tables' `artifact_skills` (the packs' effects) for the hero's window; the event told once,
  its scenario test after its boxes (guards and a text alone not built); kind 6's places from
  `resour82.def`'s 82 × 93; the window uploads a dissolve's frame whole; the Magic Well from the
  reading alone (its maximum without Intelligence, its points not on the panels).
- **M4 T2b1, the harness's writes** (*provisional*; design §5.4): the words the map's load draws
  written into the game after the staging (a monster's aggression, a chest's gold, a Tree's
  price), and a hero's Diplomacy as its level alone, without a skill slot (what a monster's
  answer reads; no screen of M4 shows the hero's skills), so that each question shows without a
  seed's search.

- **M4 T2b2, a question's facts without the engine** (*provisional*; design §5.4, *Built*): as
  T2a2's finds, each question's facts come from the map and the live part at 0 (`asks::Ask::of`):
  a dwelling's guards and hire a new game's (its growth from `CRTRAITS.TXT`; guards three weeks'
  for a creature of level 5 or more, the levels the tables' `[map.asks]`, read from the
  executable's `creature_table`); a monster's offer decided in integers (the hero's power, the
  strength step, the sympathy pairs 2 n and 2 n + 1 below 112 without Conflux's, Normal's
  difficulty, the Diplomacy the situation's: the game's float factor and 32-bit sum not
  reproduced); a Border Guard's key from the Tents' marks; a chest's experience its gold less 500
  (no Learning known); the Tomb's morale lost at its first search. M4's Part B (T3, T4) takes them
  from the engine's offers and choices instead.
- **M4 T2b2, the answers' ends** (*provisional*): a question answered tells nothing at a later
  visit; a declined question from beside ends the step at its close; an answer not built is
  refused in a recording and acts as the Cancel live; the status window built again before the
  hire's question as before the guards'; GENRLTXT 424's singular for one creature (not recorded);
  Pandora's reward boxes in the event's layout, the gain shown before the Box is taken;
  `killfade.82M` a removal sound beside the pickups; a monster drawn as its attack
  (`avwattak.def`) only when visited from beside.
- **M4 T2b2, the choices** (*provisional*): a picture chosen at the press (the frame's own message
  not read); the "or" an item of its own (id 0xFEFF); choice frames in styles 7 and 10 alone; a
  visit on its entrance looks at the mouse again as a step's end does (its place in
  `hero_move_step` not read); the tables' `chest`, `arena`, `school`, `school_price`.
- **M4 T2b2, the cursor after a walk from the recording** (*provisional*): over the view after a
  walk the original's cursor is the tile's (the pathfinder's reach), which the viewer has not (the
  arrow: TRAPS, "After a walk the mouse-over has no reach"); a sequence takes that cursor's frame
  from its recorded timeline (`Start::reached`, as `Start::points` the points since M3 T4b), so
  four of the six choices (`choose-chest`, `-chest-gold`, `-arena`, `-school`) check only when it
  changes. Live the arrow. The engine's reach (Part B) replaces it; the alternative, re-recording
  them with the mouse moved off the view before the box closes.
- **M4 T2c, a level-up's facts without the engine** (*provisional*; design §5.5, *Built*): the
  levels a visit's experience reaches from the tables' thresholds and the situation's experience
  (none given: no level, a new game's draw not known; the hero selected at 0 alone), each level's
  facts (the primary skill, the offers) from the live part's `[[level_up]]` tables in order (the
  primary skill read as the third picture from the end: the harness's item order); the dialog
  shown after a find's boxes, an answer's gain or a chest's dissolve (`Reply::after`,
  `Answer::after`); the offer learned shown only where the status window shows it (Leadership's
  morale, Luck's luck: the tables' list). M4's Part B (T3, T4) takes the facts from the engine's
  `LevelUpView` instead.
- **M4 T2c, the dialog's input** (*provisional*): an offer chosen at the mouse's press (as the
  choice boxes'); the keys 1 and 2 bound in the box's context alone (`offer-left`,
  `offer-right`); a box's close (every box's) keeps the cursor where the mouse manager last drew
  it until the next poll (recorded: `level-up-right` at 2000 and 2013). Not built: the Tree of
  Knowledge's level (its OK), a battle's (T5), the offers' right-click popups, the deadline and the
  computer's choice.
- **M4 T2c, the harness's secondary skills** (*provisional*; design §5.5): a hero's eight skills
  written after the staging (every level and slot cleared first, no selection), so that a level
  offers one advance or none without a seed's search; they and the type's cleared must not change
  the morale or luck the status window shows (TRAPS).
- **M4 T2d1, a flag raised** (*provisional*; design §5.6, *Built*): the flag raised after a find's
  first boxes and before its question, a dwelling's only without guards (its creature below level
  5); of `visit_and_update`'s redraws the minimap alone (nothing else changes in the recordings);
  the reveal of radius 3 and the scenario's test left out (nothing shows); the hint and cursor over
  the object after its visit still the owners at 0; an abandoned mine without guards keeps its
  question (its record not read); the mines' incomes as the tables' `mine_income`.
- **M4 T2d2, a boat boarded and left** (*provisional*; design §5.7, *Built*): the hero turned to
  the step at the boarding's start (both recorded heroes faced it already); a key's step's own
  path drawn as one arrow (1) on its tile in the lift's whole-screen show; points 0 at either
  visit and the hero unable to move (the Admiral's Hat not built, not recorded); MCL-Q35 as the
  original's in both profiles; the key step's tail (`stuck`) after a boat's visit alone, its
  second view draw taken for the list's dialog redraw (0x4033EA not read); left out: the
  mouse-over's boats (the live part's at 0), a computer's boarding or landing, a find on the
  tile landed on.
- **M4 T2d3, a spell cast on the map** (*provisional*; design §5.8, *Built*): the spell book left
  to M5 (no screen compared while it shows; its inputs dropped but the click that closes it);
  Summon Boat alone, from the live part's `[[cast]]` table (the engine has no adventure spells
  yet); the wait for its sound not built (with sound the original's cast ends 936 ms later); the
  window's cast button does nothing; MCL-Q14's fix (the slot shown whole) under `Bugs::Fixed` on
  every slot draw, not only after a cast; the harness's `[[cast]]` table and a hero's `spells`.
