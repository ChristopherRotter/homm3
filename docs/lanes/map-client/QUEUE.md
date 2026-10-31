# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) closed 2026-10-28** (its scope, decisions, tasks and review list:
`docs/epics/E24-archive.md`; its cuts' history: `QUEUE-ARCHIVE.md`); **M3 (the adventure
screen) refined 2026-10-28** (M3 T0, with the user: `docs/epics/E24-map-client.md`, M3's scope,
decisions and exit). Tasks in order, each about half a context; a task that grows is cut at a
seam agreed with the user. Code readings in the sandbox (`docs/exe/README.md`; every address
cited needs a map entry), their detail in `docs/epics/E24-design.md` (§4 from T2: the adventure
screen); captures by the harness (`clients/capture/README.md`), never in git. Every oddity of
the original a reading or a capture finds is an `MCL-Q<n>` in the same task (the HANDOFF's
autonomy contract). Each panel drawn leaves `h3map.py`'s `panels` mask, and `drawn` then
compares it on every situation and sequence.

T3 cut with the user (2026-10-29): T3a the minimap as a still, T3b the minimap over time (both
done: the epic's log). T4 cut with the user (2026-10-29) the same way: T4a the lists as stills,
T4b the lists over time, T4c a changed panel item drawn again alone (all done: the epic's log).
T5 cut with the user (2026-10-29): T5a the resource bar's amounts, the date and a text's reach
(done: the epic's log), T5b the status window, cut with the user (2026-10-29) into T5b1, a hero's
and a town's, and T5b2, the date's and the kingdom's (kept whole by the user; both done: the
epic's log); the other kinds (another player's turn, a resource found, a text) are M4's (the
epic's "Later"). T6 cut with the user (2026-10-30): T6a the mouse-over, cut into T6a1, T6a2a and
T6a2b (all done: the epic's log), and T6b, cut with the user (2026-10-30) into T6b1, T6b2 and
T6b3 (kept whole by the user; all done: the epic's log). T7 cut with the user (2026-10-30): T7a,
the status bar's hint unmasked and everything recorded again (done: the epic's log), beside it
M3's review by six fresh reviewers on fractions (the user: up to three at once, each under half
a context; their findings on a branch), T7b, the findings consolidated and fixed (cut with the
user, 2026-10-31, into T7b1, the redraw's and the input's, and T7b2, the rest, kept whole by the
user; both done: the epic's log), and T7c, the user's checkpoint, cut with the user (2026-10-31)
into T7c1, the review list, the questions and the hand test (done: the epic's log), and T7c2;
M4's refinement is its own task after it (M4 T0).

### E24 M3 T7c2 — MCL-Q22's and MCL-Q24's answers built, M3 archived

- **Scope:** MCL-Q22 (b) (the lane's `QUESTIONS-ANSWERED.md`): after a redraw of every dialog
  item that empties the status bar (a hero or town chosen from its list, Next Hero, a list's arrow,
  a key's step that lifts the hero, a walk's end that leaves it unable to move, the underground's
  button and key; design §4.13, §4.14) the client draws the hint of the mouse's item again;
  MCL-Q24 (b): a panel hotkey's key up during a minimap drag releases its button, without its
  command (design §4.14). Both the client's by default, the original's behind `--bugs original`
  (`h3_render::Bugs`: `sequences` and `drawn` compare with it, so every recording stays exact).
  Then M3 archived: its section (scope, decisions, tasks, exit) and its review list verbatim to
  `E24-archive.md`, a stub left with the decisions that bind later milestones (as M2's); the
  queue's M3 history to `QUEUE-ARCHIVE.md`; the HANDOFF's M3 lines trimmed. A finding of T7c1's
  hand test, if any (the log's "M3 T7c1"), goes in too or into a question.
- **Acceptance:** each answer's test fails on the old code; `h3map.py drawn` all the same;
  `clients/scripts/check.sh` all gates; the window under Xvfb in both profiles; M3 archived; M4
  T0 next. A look at both fixes in the window on the user's machine is the user's call.
- **Depends on:** T7c1 (done). **Status:** ready.

### E24 M4 T0 — M4 refined with the user

- **Scope:** M4, the engine's games shown (the epic's "Later"), refined with the user, as M3 T0
  was: what the engine gives the viewer by then (the adventure lane's increment 9 views, the
  inbox's entry; the object identity the observation lacks; `h3-protocol`'s adventure messages,
  the server lane's), the record and replay viewer, the battle handed to the battle client, the
  M3 answers' follow-ups (MCL-Q14, Q15, Q21; the epic's "Later"), the status window's turn and
  event kinds; the options, the milestone's decisions, its exit and its tasks.
- **Acceptance:** M4's section in the epic (scope, decisions, exit) and its tasks in this queue,
  agreed with the user.
- **Depends on:** T7c2. **Status:** the user's.

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-10-31 (the client lane, E17 M5 T8b2; for information, nothing to do): three changes in the
  common crates. `h3_sdl` sends `Event::Key(Key::Other)` for a key-down or key-up its map does not
  name (not in `Key::ALL`, so no key table binds it; `h3-map-view` builds and finds no command for
  it). `h3_render::text`'s `line_count` and `wrapped_width` end a second line with no place to
  break at its first glyph instead of looping for ever (a crafted font or a width of 0; every
  text that ended before measures the same). `h3_render::recolour`'s `Effect::colour` keeps its
  scale at least 1 (the same for every mask a `PixelFormat` gives).

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

- 2026-10-30 (the adventure lane, E08 T77b; **for M4, the engine's games shown**): increment 9's
  views (built by T83 to T85b, `docs/epics/E08-design.md` §12, §14): `ObjectView::owner` for every
  ownable object a viewer sees (the flag's colour), a mine's `production` for its owner's team, a
  weekly object's `visited` (the team's mark, the Garden's the viewer's own: what your
  `hover::Visit` reads from the original's memory today), a Shipyard's `boat_tile`, the added
  monsters among the objects, `GameObservation::month` and the events `ObjectFlagged`,
  `BoatBuilt`, `MonsterAdded`, `ObjectEmpty`, `WeekStarted`, `MonthStarted`, `TimedEventFired`.
  No week or month name is drawn (the original draws one for its message): naming them is the
  client's.

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
