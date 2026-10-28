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

### E24 M3 T2a — the adventure screen's dialog at a map's start, right (in progress)

Cut from T2 with the user (2026-10-28): its first half, kept whole (stills and sequences): the
reading, the dialog and the resource bar drawn as at a map's start (the buttons' states and
colours), the parked cursor, `panels` reduced to what T3 to T6 draw, the 40 situations and the 13
sequences compared again (Move hero's and next hero's changes during a walk replayed). Its
record: the epic's log, "M3 T2a".

### E24 M3 T2b — the frame's fixed cost

Cut from T2 with the user (2026-10-28): its second half, from the scope below: the frame picture
and the panels converted and drawn once a viewer, no screen copies, a frame redrawing what
changed; measured with `map-measure.cmd`'s "a frame made in" before and after on the user's
machine. **Depends on:** T2a. **Status:** ready after T2a.

### E24 M3 T2 — the adventure screen's dialog at a map's start (cut into T2a and T2b)

- **Scope:** read `adventure_full_update` 0x417420's panels (the manager's +0x44) and the
  adventure dialog +0x50: its items in the order added and drawn, each one's id, place, picture
  or DEF and states; the buttons (`iam000` to `iam015`), the status bar (`AdRollvr.pcx`), the
  resource bar (`AResBar.pcx`), the status window's and the lists' frames as at a map's start;
  `h3_map_view::layout` gains the dialog (`h3_ui::ui`'s format), the screen drawn with it. The
  frame's fixed cost (the inbox's, `QUEUE-ARCHIVE.md` 2026-10-27, E24 M2 T4b3): the frame
  picture and the panels converted and drawn once a viewer, no screen copies, a frame redrawing
  what changed; measured with `map-measure.cmd`'s "a frame made in". The parked cursor drawn
  (the mask's). Recorded: the situations' screens with the panels; `panels` reduced to what
  T3 to T5 draw.
- **Acceptance:** the static panels exact on the 40 situations and the 13 sequences; a frame's
  fixed cost measured before and after; the Definition of Done.
- **Depends on:** T1a (merged, #545). **Status:** ready (T1b's pull request merged first, or
  `master` merged in after it: both touch `h3-ui`).

### E24 M3 T3 — the minimap

- **Scope:** `minimap_draw` 0x412C40 read (13 callers) and built: each tile's colour (the
  terrain's DEF's palette entry 22, design §1.8), the owners' colours, the unexplored tiles, the
  view's box, the map's size scaled to its square, the underground's; redrawn at a scroll, a step
  and a selection as the original does (its callers). Recorded: the sequences show it scrolling.
- **Acceptance:** the minimap exact on every situation and sequence (`panels` without it);
  the Definition of Done.
- **Depends on:** T2. **Status:** ready after T2.

### E24 M3 T4 — the hero and town lists

- **Scope:** read and built: the hero list (a hero's small portrait `HPS*.pcx`, the movement and
  mana bars `imobil.def` and `imana.def`, the selection's frame, the scroll arrows, empty slots)
  and the town list (`ITPA.def`: a town's picture by its kind, fort and the build of the day; the
  selection, the scroll); `live` gains the viewing player's heroes and towns in their lists'
  order, a hero's movement and mana and the one selected (`h3mem.py`, additive). Situations with
  more heroes and towns than the lists' slots.
- **Acceptance:** both lists exact on every situation and the new ones; the Definition of Done.
- **Depends on:** T2. **Status:** ready after T2.

### E24 M3 T5 — the status window, the resource bar and the date

- **Scope:** read and built: the status window's kinds (`AdStatHr`, `AdStatCs`, `AdStatOt`,
  `ADSTATIN`, `ADSTATNX`: a hero's portrait, skills, morale, luck and army; a town's; the
  kingdom's and the date's) and when each shows; the resource bar's seven amounts and the date
  (the fonts, the string catalogue's keys from the original's tables); `live` gains the
  resources and the date. Situations for each kind.
- **Acceptance:** the status window, the resource bar and the date exact on every situation and
  the new ones (no `panels` mask left but the status bar's hints); the Definition of Done.
- **Depends on:** T2 (T4 for a hero's or a town's army and portraits). **Status:** ready after T4.

### E24 M3 T6 — the mouse, the keys during a walk, the sound without the focus

- **Scope:** `adventure_mouse_move` 0x40E360's unread part and `view_mouse_tile` 0x40DEB0 (the
  8-pixel question, design §1.8): the cursor over the map by its tile (`cradvntr.def`'s kinds;
  the reach and its days given: a recording's from the original's memory), the status bar's
  hints over the map's objects and the panels (the catalogue's keys); the view-only clicks (a
  hero or a town chosen, the lists scrolled, Next Hero, the underground, the minimap, the status
  window) and the other buttons' press; the panels' hotkeys through the key table; MCL-Q10's
  keys during a walk (Space's cancel, the input ring's bound of 63); the sound without the focus
  (`QUEUE-ARCHIVE.md` 2026-10-27, E24 M2 T6a2: the rest read, the window by
  `Window::focused()`). Recorded: sequences with the clicks and the cursor's kinds.
- **Acceptance:** the new sequences as recorded, every frame; the clicks in the window; the
  Definition of Done.
- **Depends on:** T3 to T5. **Status:** ready after T5.

### E24 M3 T7 — M3's closure

- **Scope:** fresh reviewers (one per area), their fixes; every situation and sequence recorded
  again; the review list with the user; the window cross-built for the user's machine and a
  hand test (`clients/out/win/`, a `.cmd` with the checks on screen and a notes file); M3
  archived; M4 refined with the user.
- **Acceptance:** the review list accepted; the hand test passed; M3 archived; M4's tasks queued.
- **Depends on:** T1a to T6. **Status:** waiting (the user, at its checkpoint).

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

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
