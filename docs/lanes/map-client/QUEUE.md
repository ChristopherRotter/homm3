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
and their fixes (done 2026-10-27: 24 of 29 fixed; `E24-log.md`), its five music findings cut out
as T6a2 (the user; done 2026-10-27, whole), then T6b, the closure, cut at its seam before it
started (the user, 2026-10-27): T6b1, the recordings again and R2-8 (done 2026-10-27: every
sequence and still exact; R2-8 as #523), then T6b2, the user's, cut at its seam (the user,
2026-10-27, away from the machine): T6b2a, the review list and the questions (done 2026-10-27:
the review list accepted, MCL-Q7, Q8 and Q12 answered, a build and a checklist on the user's
machine), then T6b2b, the answers built, and T6b2c, the hand test and M2 archived.

### E24 M2 T6b2b — the answers built: MCL-Q7, Q8 and Q12

- **Scope:** the three answers of M2's closure (the lane's `QUESTIONS-ANSWERED.md`), each the
  client's by default and the original's behind `--bugs original` (`h3_render::Bugs`, as
  MCL-Q2 to Q6): MCL-Q7, the lifted hero cut at the map's side moved by the offset in passes 5,
  7 and 9 (design §3.8); MCL-Q8, a set of the 70 ambient loops instead of 32 bits (design
  §3.11); MCL-Q12, a track asked for during its own fade-out kept (`h3_map_view::music`'s
  filter: Grass resumes where its fade began). The design's paragraphs say which profile does
  what.
- **Acceptance:** each case tested in both profiles (each new test failing on the old code); the
  13 recorded timelines line by line and `h3map.py drawn` the same (no recorded sequence meets
  these cases); the Definition of Done.
- **Depends on:** T6b2a. **Status:** ready; the sandbox's work, no user needed.

### E24 M2 T6b2c — the hand test and M2 archived

- **Scope:** the window by hand on the user's machine: `clients/out/win/map-try.cmd` (the T6a
  fixes: Alt+Tab with the pointer at an edge, fullscreen's bars, F7 in a situation; T6a2's
  music by ear in `seq-walk`), the user's notes in `map-try-notes.txt` beside it; T6b2a put
  there the build of `master` at #527: build again after T6b2b (`--target
  x86_64-pc-windows-gnu`) and change item 2c's text (Grass comes back) before the user tries it,
  unless the notes are already written. Each finding checked (fixed, queued or an `MCL-Q<n>`);
  then M2's section moved verbatim to `E24-archive.md` (scope, decisions, task list, the
  accepted review list; a row and a pointer left) and the HANDOFF's M2 lines trimmed.
- **Acceptance:** no finding of the hand test left unrecorded; M2 archived; the Definition of
  Done.
- **Depends on:** T6b2b. **Status:** the user at the machine (or the notes file written).

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
- **Depends on:** T6b2c (M2 archived). **Status:** the user present.

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
