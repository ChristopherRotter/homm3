# Map client lane — QUEUE

Pre-refined tasks of the map client lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

Epic: E24, the adventure map client. **M1 (stills of the map view) closed 2026-10-24**; **M2
(the map moving) refined 2026-10-26** (M2 T0, with the user: `docs/epics/E24-map-client.md`, M2's
scope, decisions and exit; the recording's format there). Tasks in order, each about half a
context; a task that grows is cut at a seam agreed with the user. Code readings in the sandbox
(`docs/exe/README.md`; every address cited needs a map entry), their detail in
`docs/epics/E24-design.md` (§3 from T1a: the map moving); captures by the harness
(`clients/capture/README.md`), never in git. Every oddity of the original a reading or a capture
finds is an `MCL-Q<n>` in the same task (the HANDOFF's autonomy contract).

T1a was cut at its seam before it started (the user, 2026-10-26): T1a1 the recording and the map
standing still, T1a2 the scroll; T1b, the walk, done whole (all three done 2026-10-26:
`E24-log.md`); T2, the map over time headless, done whole (2026-10-27: the user chose it over a
cut before the frame check); T3 cut at its seam before it started (the user, 2026-10-27): T3a,
the walk's timing headless, and T3b, the walk drawn (taken whole), both done 2026-10-27. T4 cut
at its seam before it started (the user, 2026-10-27): T4a, the window replaying and the key
table (done 2026-10-27), then T4b, the map's live input and the Windows build, cut at its seam
before it started (the user, 2026-10-27): T4b1, the live input in the sandbox (done 2026-10-27),
then T4b2, the Windows build on the user's machine (done 2026-10-27), which found the view's
drawing too slow: T4b3 (the user, 2026-10-27; done 2026-10-27). T5, the sound, done whole 2026-10-27 (the user
chose it over a cut).

T6, M2's closure, cut at its seam before it started (the user, 2026-10-27): T6a, the reviewers
and their fixes, then T6b, the closure.

### E24 M2 T6a — the reviewers and their fixes

- **Scope:** three fresh reviewer agents in parallel, one per area (the map over time and its
  drawing; the window, the input and the sound; the harness and the recordings); each finding
  checked against the code and, where it names the original, the executable; fixed with a test,
  or queued with its reason; the findings and their outcome in the log, the review list's T6a
  entry.
- **Acceptance:** the lane's scoped verify without a failing step, CI green; the Definition of
  Done.
- **Depends on:** T1a1 to T5 (T4b1 to T4b3). **Status:** in progress (2026-10-27).

### E24 M2 T6b — M2's closure

- **Scope:** M2's exit (the epic) checked on `master` after T6a's fixes: the 13 sequences
  recorded again or compared again (`frames --reuse`), every line and frame; R2-8's one-line
  `cross-lane` pull request if no task took it (the inbox); the review list to the user; M2's
  section moved to `E24-archive.md`; M3 T0 queued (the common crate proposed to the client lane
  first: the epic's M2 decision).
- **Acceptance:** the review list accepted by the user; the Definition of Done.
- **Depends on:** T6a. **Status:** ready once T6a merged.

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

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

- 2026-10-24 (this lane, E24 M1 T6b's reviewer, R2-8; with the next writer pull request): the
  writer's edge bound for an object (`crates/h3-map/src/writer/extras.rs` `PAST_RIGHT`, `PAST_BOTTOM`, its test
  at 799) lets an anchor lie 7 past the map's side, where an 8-wide template covers no cell on
  the map: one too generous, its comment off by one (6 and 4, or the comment). The adventure
  lane's crate: a `cross-lane` change, so it waits for the next addition the situations need.

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
