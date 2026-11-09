# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) closed 2026-10-28**; **M3 (the adventure screen) closed 2026-10-31** (their
scope, decisions, tasks and review lists: `docs/epics/E24-archive.md`; their cuts' history:
`QUEUE-ARCHIVE.md`). **M4 (the engine's games shown) refined 2026-10-31** (M4 T0, with the user:
`docs/epics/E24-map-client.md`, M4's scope, decisions and exit): Part A (T1, T2) the original's
reactions to a game's events, its memory the live part; Part B (T3, T4) the engine's games
through `h3-protocol`'s whole-game record (ADR-0020 point 8), after the server lane builds it;
Part C (T5) the battles handed to the battle client's viewer. Tasks in order, each about half a
context; a task that grows is cut at a seam agreed with the user. Code readings in the sandbox
(`docs/exe/README.md`; every address cited needs a map entry), their detail in
`docs/epics/E24-design.md` (§5 from T1: the game's events); captures by the harness
(`clients/capture/README.md`), never in git. Every oddity of the original a reading or a capture
finds is an `MCL-Q<n>` in the same task (the HANDOFF's autonomy contract).

### E24 M4 T1 — the turn's change

- **Scope:** read and build what the original shows when a turn changes, the live part from its
  memory (`h3mem.py`, additive): End Turn's click (the button's action, its question when a hero
  can still move), the turn's end and the next player's start; the new day (the date's step,
  built at M3 T5b2, from its cause), the week's and the month's start and their messages (the
  week's and month's names are the client's to draw: the engine sends `WeekStarted` and
  `MonthStarted` with the kind only; a comparison reads the original's from its memory); another
  player's turn: the status window's kind 5 (`AdStatNx.pcx`, `crest58.def`, `HourSand.def`,
  `HourGlas.def`, 0x453250; design §4.8) with its tile, hint and cursors (0x40DEB0, §4.10), and
  a computer player's moves where the viewer sees them (the view's scroll to them, their walk at
  the computer's speed, `enemyMoveTime` in VCMI's terms; read, not assumed). New sequences
  recorded in the original (a day's end, a week's and a month's start, a computer player's turn;
  the inbox's harness change first: `setup.sh` again).
- **Acceptance:** the readings in a new `E24-design.md` §5; the new sequences exact on the whole
  screen and `h3map.py drawn` all the same; `clients/scripts/check.sh` all gates; every oddity an
  `MCL-Q<n>`.
- **Depends on:** T0 (done). **Status:** next (likely cut at its start with the user).

### E24 M4 T2 — a visit's results

- **Scope:** read and build what the original shows when a visit or an event changes the game,
  the live part from its memory: the adventure's message box (its dialog: the text through the
  catalogue, the pictures of what was found, OK and Yes/No, its sounds, its place) for the finds
  and questions the engine's visits make (a pile, a chest's or a School's choice, a guard's
  question, an event's or a sign's text, creatures joining, a level-up's choice); the status
  window's kind 6 (a resource found, 0x451220, 5 seconds) and kind 7 (a text, 0x451820); an
  object taken or a monster beaten (the map, the minimap and the panels redrawn); a flag raised
  (the owner's colour on the object and the minimap); a boat boarded and left; a spell cast on
  the map recorded (MCL-Q14: if the stale mana bar shows, the client shows the slot whole, the
  original's behind `--bugs original`). New sequences recorded in the original, the harness
  giving the answers.
- **Acceptance:** as T1's.
- **Depends on:** T0 (independent of T1). **Status:** waiting (likely cut at its start with the
  user, by dialog family).

### E24 M4 T3 — a record's start

- **Scope:** a game's record (the protocol's whole-game messages for one player) read: the
  static art from the map it names; `h3_map_view::live` from its first observation through an
  adapter from the protocol's types (*provisional*): each object by its identity, a random
  object's type, the owners, the heroes (class, owner, tile, boat; their facing from their
  steps), the empty boats, the explored tiles, the kingdom (the heroes' portraits, movement,
  mana and whether each can move, the towns, the resources, the date), a hero's status, the
  hover's facts (increment 9's views: the owners, visited marks, a mine's production); a town's
  own name from the map. MCL-Q15 (b): an amount past the bar's seven digits written so that it
  fits (its form a review item), the original's behind `--bugs original`. MCL-Q21 (b): the
  viewer's load refuses a map the engine's import refuses for it. The check: the engine's start
  drawn as the original's on the maps both play (the writer's without random objects, the 8
  shipped maps that import whole), a situation from each.
- **Acceptance:** the starts equal to the original's; `drawn` all the same; `check.sh` all gates.
- **Depends on:** the server lane's whole-game record (E16 M5's start: its inbox, 2026-10-31);
  the adventure lane's object identity and increment 9's views (its inbox, 2026-10-31; T83b2b,
  T85). **Status:** waiting.

### E24 M4 T4 — a record played

- **Scope:** a record's messages played over time: each event as T1 and T2 built it (the steps
  along the engine's paths as M2's walk, the reveals, the removals, the owners, the turns and
  days, the message boxes with the record's answers, the other players' turns as a computer
  player's); the viewer's controls (pause, step, restart, a speed: the window's keys, new design,
  each a review item). The check: scripted games (the same commands, no random draw between
  them) played in the engine and in the original, compared frame by frame, the original's
  visual moment from its recording; a golden adventure replay as a record viewed to its end, its
  battles skipped to their results until T5.
- **Acceptance:** the scripted games exact; `drawn` all the same; `check.sh` all gates.
- **Depends on:** T1, T2, T3. **Status:** waiting.

### E24 M4 T5 — the battles handed over

- **Scope:** a battle of the game drawn by the battle client's viewer in the map viewer's
  window, as the original switches screens (its sounds and music), and the map back with the
  result applied; the battle's background and colours from the game (CLT-Q4, CLT-Q6:
  `BattleSetup.background`, `colours`, set by `h3-game`).
- **Acceptance:** a golden replay's battles shown in the window; the switch compared with the
  original's recording where it is drawn; `check.sh` all gates.
- **Depends on:** T4; the client lane's library entry (its inbox, 2026-10-31). **Status:**
  waiting.

### E24 M4 T6 — M4's closure

- **Scope:** fresh reviewers (one per area) and their findings fixed or listed; M4's review list
  and its questions for the user; a hand test on the user's machine (`clients/out/win/`: a
  golden replay viewed with its battles; the map's sound by ear on a whole map, the user at M2
  T4b2: the transitions sounded unclean; MCL-Q11's `HORSE20` to `HORSE29` listened to for its
  (c)); M4 archived.
- **Acceptance:** M4's exit (the epic) met; the review list accepted by the user.
- **Depends on:** T1 to T5. **Status:** waiting.

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-11-09 (the adventure lane, E08 T104, branch `adventure/e08-t104`; **a refusal, for the
  viewer's map list**): the game's build now refuses a teleporter with a visitable tile off its
  surface (`h3_adventure::SetupError::TeleporterSurface(object index)`, A-42, `ADV-QUIRK-44`): a
  monolith's or a Subterranean Gate's on water or rock, a Whirlpool's on land or rock. None of
  the 160 shipped maps is refused; a crafted map the original may load is. The channels and the
  gates' pairs are the static map's (`StaticMap::channels`), the server's alone: never a view's
  (ADR-0016 point 6). Nothing to act on before the jump's events (T105a).
- 2026-11-07 (the adventure lane, E08 T103a, branch `adventure/e08-t103a`; **the teleporters'
  types, for when E24 draws them**): `h3_adventure::ObjectKind` has three new variants,
  `Monolith { channel: u8, way: MonolithWay }` (`Entrance`, `Exit`, `TwoWay`; the original's classes
  43 to 45, the channel its subclass), `SubterraneanGate` (103) and `Whirlpool { divisor: u32 }`
  (111); the packs' types `base:monolith_{entrance,exit,two_way}_0` to `_2`, `sod:…_3` to `_7`,
  `base:subterranean_gate`, `base:whirlpool` (adventure ruleset part v18). The import maps the
  classes to them in T103b (map import v21); until then they import as blockers. No view changes:
  a viewer sees the type as for any object (ADV-FOG-004); a jump's events come with T105a.
- 2026-11-07 (the town lane, E09 T14b, branch `town/e09-t14`; the capture harness, informational):
  `h3adv` plays a walk's `actions` on the town screen when it opens (then closes what they left
  open), with the new actions `hall = true` (a click on the shown hall building, items 10 to 13: the
  hall, vtable 0x6437A0) and `slot = <building id>` (the hall's slot showing it: the building's
  dialog, 0x643944; `click = ["build"]` its Build 30722, `cancel` 30721), and the reads
  `town:x,y:mark`, `hall:slot:<id>`, `build:button`, `build:text`, `screen`, `town_screen_closed`;
  `h3mem`'s town record gains `mark` (+0x2), `h3lod` gains `building_names`. A walk without
  `actions` behaves as before. Learned: the panel's hall picture opens nothing; a map whose players
  are all allies ends at a visit's close (an enemy is needed). The harness check passed after.
  `clients/capture/README.md`, "The hall and the build".
- 2026-11-07 (the client lane, E17 M6 T6a, branch `client/e17-m6-t6a`; for you to take or leave): `h3-view`'s `tests/purity.rs` took your R2-15 fixes, and its reviewer found two more holes your copy (`clients/crates/h3-map-view/tests/purity.rs`) shares: `attributes` closes an attribute at the first line ending in `]`, so a `]` in a comment inside a multi-line `#[allow(...)]` hides the lints after it (now: `code_of` cuts the `//` comment outside string literals and the brackets are counted), and `deny_block` matches a commented-out `// #![deny(` (now: read from `attributes`' entries). `h3-view`'s copy has both fixes and their tests.
- 2026-11-07 (the adventure lane, E08 T100b, branch `adventure/e08-t100b`; the capture harness, informational): `h3adv` gains a `key` step's `repeat = { most, first, until = { with } }` (the key pressed again and again, a row each: `presses`, `presses:<columns>`, `presses:changed`), a recipe's `[places]` (`hero:place`, `hero:at`), a step `look = "<hero>"` (the view centred by the minimap, nobody selected; its `before:` reads before the scroll), `walk = "off"` (the first open neighbour), `only_if`'s `empty` and `occupied`, and the observations `whirlpools` (`h3mem.whirlpools`: game +0x4E77C's vector, begin and end at +0x4E780), `tile:x0,y0..x1,y1`, `tile:from`, `tile:here`, `tile:hero[<name>]`, `answers`, `selected`, `target`. No behaviour of an existing step changes. Observed: the selected hero is lifted off its tile's record after its move and at its turn's start, and a scroll of the view puts it back (a tile read depends on the last scroll). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".
- 2026-11-06 (the adventure lane, E08 T100a, branch `adventure/e08-t100a`; the capture harness, informational): `h3adv` gains a step's `replay` (Replay Opponent Turn: the panel's Adventure Options button, the map dialog's item 9; its dialog vtable 0x63A610, item 5; played on the frozen clock, stepped 20 ms, the flag 0x696A04; `dumps = "every"`) and `at_opening` (reads as a step's first battle opens), `only_if = { at = [x, y] }`, `again_if = { marks = "any" }`, and the observations `replay`, `replay:views`, `replay:watched`, `replay:options`, `view:before`, `view:after`, `view:size`, `explored:x0,y0..x1,y1`, `at_opening:`, `battles`, `exchange`, `army:rows`, `hero:in_boat`. Behaviour changes: a `key` step plays what the key brings by the step's `answer`, `battle` and `level_up` (it stopped at a battle before); a skipped step leaves a field another step filled (it wrote "not played" over it). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".
- 2026-11-06 (the town lane, E09 T8c1 and T8c2, `town/e09-t8c1` (#750, merged) and
  `town/e09-t8c2`; **for the map's towns, when E24 draws them**): the observation carries them.
  `h3_adventure::ObjectView::town: Option<TownId>` links a seen object to its town;
  `h3_game::GameObservation::towns: Vec<TownView>` (in `TownId` order: the towns a tile of which
  the viewer has explored; one only on the fringe keeps its object's `town` but has no view)
  holds the hint's and the right-click popup's facts: `owner`, `faction`, `name` (none: the
  client's default, `h3_towns::TownStatic::name`), `fort`, `capitol`, `garrison: GarrisonView
  { level, stacks, army }` (the popup's level 0 to 3; the stacks packed in slot order, the word
  from level 2, the count at 3, as `quick_view_army` 0x530990 places them; `army` by slot to the
  owner's team only) and `owner_facts: Option<TownOwnerView { hall, income }>` at level 3 (the
  owner's team, or a hero of the viewer's with a Rogue whose Visions reach the town's entrance,
  T8c2). What each viewer sees: `docs/06-security.md`, "Towns"; the popup: survey
  `E09-towns.md` §12.3; design note `E09-design.md` §12, §17.

- 2026-11-04 (the client lane, E17 M6 T4b2a, branch `client/e17-m6-t4b`; the common crate
  `h3-ui`, a behaviour change, `cross-lane`): the user decided (2026-11-04) that the windows' own
  pause, a frame on, restart and vsync move to Ctrl+F5, Ctrl+F6, Ctrl+F7 and Ctrl+F8 in both
  viewers, because the original's battle takes F5 to F8 alone (F5 Creature Info's level, F6 to
  F8 the grid and shadows). `h3_ui::keys::Window` gains `ctrl` and `chord` (the default chord;
  `key` stays the key's name); `h3_map_view::keys`' defaults take `w.chord()`, its tests and the
  usage name the new chords (built on that branch). The music, the effects, the cursor's kind,
  fullscreen and Esc are unchanged; a user's `keys.toml` still overrides any. Informational
  unless `docs/epics/E24-design.md` §3's keys bullet (F5 pause, F6 the next frame, F7 restart, F8
  vsync) should say so: the map lane's file.

- 2026-11-04 (the adventure lane, E08 T94a, `adventure/e08-t94a`; **your M4 T0 request (2), for the random dwellings**): a random dwelling not tied to a town (216 to 218) resolves when the game is built (the setup stream, before every other draw of the static map): its `ObjectView::object_type` is the drawn dwelling type, its `anchor` the original's moved one (the type's shift: only the Angel Portal's, 1; it may lie one column past the map's right edge, x == width, where only its free cell is) and `blocked`/`visitable` the type's footprint, its `owner` the map's; nothing tells a viewer it was random. One tied to a town stays an untyped blocker until E09 (ADV-STUB-13).
- 2026-11-03 (the adventure lane, E08 T90, branch `adventure/e08-t90`; the capture harness, informational): `h3adv` and `h3mem` gain the dwellings' records (`dwelling:x,y`, game +0x4E39C), `teams`, `objects:` and `access:` (a rectangle's map objects with class and subclass, its tiles' blocked and visitable bits), `camp:x,y`, the hire dialog as a screen (vtable 0x640C4C; `hire`, `hire:opening`, `hire:after`; actions `click = ["max", "recruit"]`, `amount = <n>`), a step's `hint = [x, y]` and `popup = [x, y]`, `opening:defender`, `ai_turns`. Behaviour change: `end_turn` now waits out a computer player's turn (it returned during it); nothing changes on maps with humans only. `clients/capture/README.md`, "Dwellings, the hire dialog and a computer player's turn".

- 2026-11-03 (the town lane, E09 T3b, branch `town/e09-t3b`; the capture harness, informational
  unless you play battles with `attack`): `h3adv` gains the town reads (`town:x,y:owner`,
  `visitor`, `built`, `every`, `garrison`, `slot:N`; `towns:every_vs_built`,
  `towns:owned_garrisons`), `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
  `unknown_screens`, a `new_game = true` step, the town screen (vtable 0x64372C, closed by item
  30720) and the level-up dialog (0x63FE60, `level_up`) as known screens, and a list of creatures
  in the table battle plan's `attack`. Behaviour changes: `play_turns` reads the turn again after
  its 0.5 s wait (an order went into the next stack's turn), a retreat waits out the retreating
  side's Catapult, a `text:` literal ending in `: ` joins the next value without `; `, and `batch
  --map` fills the template beside the map when its stamp matches (no cargo build). A town's
  visitor (+0x10) reads −1 while its hero is the selected one. The harness check passed before and
  after. `clients/capture/README.md`, "Towns (E09 T3b)".
- 2026-11-02 (the battle lane, E26 T5; for a computer side in your battles, when you add one):
  call the fair system actor `h3_battle_ai::choose_fair` by default (ADR-0027 point 4 as
  amended; the user, AI-Q10), the all-seeing `choose` only as a game's option.

- 2026-11-02 (the client lane, E17 M6 T2a1; for information, nothing to do): additions to the
  common crates for the battle's stack stats dialog (`docs/epics/E17-design.md` §12.1):
  `h3_render::blit::draw_frame_clipped`; `h3_ui::ui::Def`'s `group`, `creature` and `frames_of`
  (serde defaults: a layout without them reads as before) and the
  `ui::Draw` variants `Creature` (a creature DEF item, the flags 0x12: the shadowed drawer, its
  canvas placed by group 2's first frame, cut to the item) and `Shared` (another DEF's frame in
  the item's DEF's colours: the executable keeps frames by name, CLT-Q36), both drawn by
  `h3_ui::paint` (`names`, `draw`, `reach`; `creature_frame`). Every existing draw is the same;
  a `match` on `ui::Draw` needs the two arms.

- 2026-11-02 (the battle lane; for your information): `h3_formats::text::TextTable` now makes
  each run of quotes in a quoted field one, as the executable's text-table readers do
  (`installation.md` §4): a text `h3-map-view` reads from a table shows single quotes where the
  game does.

- 2026-10-31 (the adventure lane, E08 T83b2b; **for E24 M4 T3**, the "increment 9's views" it
  waits on): `ObjectView` carries `owner` (every viewer that sees the object), `production` (a
  mine's resource and daily amount, its owner's team only), `guards` (`GuardsView`: `creature`
  when all stacks are of one, none for the original's generic text; `count_word` of the total
  count, every stack counted; `army`, the stacks, for the owner's team only), `visited` (a weekly
  object's mark, the viewer's own) and `boat_tile` (a Shipyard's); it is no longer `Copy`. The
  object identity, the random objects' drawn types and MCL-Q21 (b) of your note stay open in the
  adventure lane's inbox; the month's kind comes with T85.

- 2026-10-30 (the client lane, E17 M5 T8b1, reviewer R7's fixes; **your harness changed**):
  after it merges, run `clients/capture/setup.sh` again for every run directory you use: the
  hook DLL's ack has ten fields now, and an older `xdd.dll` stops a run with "stale xdd.dll".
  `H3_RUN` without `WINEPREFIX` is refused (both scripts); the run directory is resolved
  absolute and `Game.start` fails unless the prefix's `d:` points at it. A run also fails when
  the ack's clip-clock field is not 1, when a capture reaches a result window with an empty
  `VIDEO.VID`, and in a sound run when a sample could not be parsed or more than 32 handles
  played. Sound runs: the battle's start sound must be a new sample still playing at the
  opening (else the run raises), loops counted (`sounds_settled` too); a paused sample answers
  Miles' 8; `wineserver -w` waits at most 60 s. Nothing else drives the original otherwise
  (runbook, "Sound", "The clips", "Traps").

- 2026-10-23 (the client lane, E17 M3 T9c, at the split): moved here from the client lane's
  inbox, the adventure lane's entry of 2026-10-21 (E08 T30, decision 65, ADR-0017 point 6 amended
  in #353; for the hot-seat client, E24 M5): **a level-up holds the game.**
  While a hero's level-up with two offers is open, the engine accepts no command but its owner's
  answer (`h3_game::GameState::waiting_for` names the owner; `level_up(owner)`, or the owner's
  `GameObservation.decision` as `DecisionView::LevelUp`, gives the hero, level, primary skill and
  the two offers; `Answer::First`/`Second` answer it). A battle's defender that won answers it
  outside its turn. The user's requirement for hot seat: when the player who must act changes,
  show a handover screen that hides everything (map, hero list, the dialog) until that player
  confirms it is in control, then the level-up dialog alone, and hand back to the player whose
  turn runs the same way, so no one sees another player's view.

Archive of the lane's finished and informational entries: `QUEUE-ARCHIVE.md` (grep it for an id or a task a doc cites). Keep this file to what is open: a done task leaves it, a notice nobody has to act on goes to the archive at once (`docs/08-work-process.md`, "Keeping the start-of-session reading small").
