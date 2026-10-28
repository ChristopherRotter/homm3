# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) closed 2026-10-28** (its scope, decisions, tasks and review list:
`docs/epics/E24-archive.md`; its cuts' history: `QUEUE-ARCHIVE.md`). Tasks in order, each about
half a context; a task that grows is cut at a seam agreed with the user. Code readings in the
sandbox (`docs/exe/README.md`; every address cited needs a map entry), their detail in
`docs/epics/E24-design.md`; captures by the harness (`clients/capture/README.md`), never in git.
Every oddity of the original a reading or a capture finds is an `MCL-Q<n>` in the same task (the
HANDOFF's autonomy contract).

### E24 M3 T0 — M3 refined with the user

- **Scope:** first the common crate, proposed to the client lane (its inbox, since T6b2a): one
  crate of the client workspace both lanes share, holding `h3-view`'s `clock`, `cursor`, `ui`,
  `text`, `layout` and the parts of `mouse` that are not the battle's, and `h3-map-view`'s
  `keys` (both viewers' keys through it) and perhaps `music`; one `shared/` pull request for
  `lanes.toml`, then the move, `h3-map-view`'s copies gone (the epic's M2 decision). Then M3 with
  the user: its scope, decisions and exit (the epic's "Later": the minimap, the hero and town
  lists, the status window, the resource bar and the date, the buttons, the mouse and its
  cursor; the string catalogue's keys), MCL-Q10's keys during a walk (Space's cancel, the
  input ring's bound), MCL-Q9 if still open, the inbox's frame cost and the sound without the
  focus placed; M3's tasks queued.
- **Acceptance:** M3's section refined in the epic; the common crate agreed with the client lane
  (or its question asked); M3's tasks queued.
- **Depends on:** M2 closed (2026-10-28). **Status:** the user present.

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

- 2026-10-27 (this lane, E24 M2 T6a2): **the sound without the focus** is not built. The original
  on losing it (`window_procedure` 0x4F7E88, `sound_focus_lost` 0x599C40, read in part) stops
  every effect sample and fades the music out (the stub 0x59A830: `music_stop_fade`, its place
  kept); on regaining it `music_resume` 0x59AC00 asks for the track again (it fades in where it
  faded out) and the effects come back by 0x4F7E2E (not read). The window plays on without the
  focus. With M3's settings or the next sound task: read the rest, then `h3_map_view::music`'s
  model takes a fade without a pending track and the window follows `Window::focused()`.

- 2026-10-27 (this lane, E24 M2 T4b3; the user: "it will do for now, but we should check back in
  at some point"): **a frame's fixed cost.** A frame takes 4 to 6 ms to make on the user's machine
  (5.7 in the sandbox), about 4 even for a near-empty view: the frame picture's and the terrain's
  pixels drawn every frame, the widening to 32 bits, the painter's two copies of the screen, the
  scene built afresh, the objects' transparent pixels still visited (design §3.12, "Left"). Take
  it up with the next work that touches the drawing (M3's panels at the latest): spans of a
  frame's drawn pixels made once, the frame picture converted once a viewer, no screen copies, a
  scroll or a walk redrawing only what moved; measure with `map-measure.cmd`'s "a frame made in".

- 2026-10-26 (the adventure lane, E08 T57; for information, nothing to do): the harness's
  reads and steps for batch E7 (`clients/capture/README.md`, "Rewards on the map"). Behaviour that
  changed for every recipe: `before:<observation>` is now read right before the step (any
  observation, tiles too; it was the step's snapshot), a message box whose OK waits for one of two
  pictures is answered by `choice` (it stopped the batch before), and each message closed with its
  OK keeps its pictures (`message_pictures`). New: `h3mem.object_at`, `universities`,
  `dialog_pictures`, `open_tiles`, `map_kind`, a hero's spell points, bonuses and status flags, a
  stack's army slot; `h3adv`'s `each`/`select` grids, list records and `+` fields, `once`,
  `days`, `near` and `[[require]]`. The harness check gained Win's chest and Warrior's Tomb.

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
