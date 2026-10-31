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
a context; their findings on a branch), T7b, the findings consolidated and fixed, and T7c, the
user's checkpoint.

### E24 M3 T7b — the review's findings consolidated and fixed

- **Scope:** the 42 findings of T7's six fresh reviewers (the branch
  `map-client/e24-m3-t7-review`, `docs/lanes/map-client/review-m3/`: its README's summary first,
  then each file), each one checked and then fixed, queued for M3's review list, turned into an
  `MCL-Q<n>` or declined with why, the bugs and divergences first (F2-1, F4-1 to F4-5, F3-2); the
  captures a fix changes recorded again (`drawn` first: the runbook, "Which captures a change
  needs"); MCL-Q25's and MCL-Q26's answers if the user gave them. The findings' folder is not
  merged: the log keeps each finding's outcome. Cut with the user at its start if it outgrows half
  a context (by area: the bugs and divergences, then the robustness and the rest).
- **Acceptance:** every finding has an outcome in the log; `drawn` all the same after the fixes'
  captures; the scoped verify and `check.sh` pass.
- **Depends on:** T7a. **Status:** ready.

### E24 M3 T7c — M3's review list, the hand test, M3 archived, M4 refined

- **Scope:** the review list with the user (the epic's "M3's review list", T7b's additions); the
  window cross-built for the user's machine and a hand test (`clients/out/win/`, a `.cmd` with
  the checks on screen and a notes file; MCL-Q18's fix among them); M3 archived (its section
  verbatim to `E24-archive.md`); M4 refined with the user.
- **Acceptance:** the review list accepted; the hand test passed; M3 archived; M4's tasks queued.
- **Depends on:** T7b. **Status:** the user's checkpoint.

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
