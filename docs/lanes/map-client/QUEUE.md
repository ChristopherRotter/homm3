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
