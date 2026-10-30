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
T6b3 (kept whole by the user; all done: the epic's log).

### E24 M3 T7 — M3's closure

- **Scope:** fresh reviewers (one per area), their fixes; the status bar's hint unmasked everywhere
  (`masks.toml`'s `hint` into `panels`, `hinted` gone; moved here from T6a2 at its cut) and every
  situation and sequence recorded again; the review list with the user; the window cross-built for the user's machine and a
  hand test (`clients/out/win/`, a `.cmd` with the checks on screen and a notes file); M3
  archived; M4 refined with the user.
- **Acceptance:** the review list accepted; the hand test passed; M3 archived; M4's tasks queued.
- **Depends on:** T1a to T6b3 (all done). **Status:** ready (the user, at its checkpoint; its
  reviewers also take MCL-Q25's and MCL-Q26's answers if given).

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

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
