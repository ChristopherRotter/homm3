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
standing still, T1a2 the scroll (both done 2026-10-26: `E24-log.md`).

### E24 M2 T1b — the walk, read and recorded

- **Scope:** readings (`E24-design.md` §3): `hero_move_step` 0x4805E0's animation (E08 read its
  pricing only), the moving hero's passes (`view_moving_hero_piece` 0x47F860, its shadow
  0x47FB40, `view_moving_hero_step` 0x47FC80: the group +0x1FC and frame +0x200, the pixel offset
  per frame, the frames per step), the time per frame at each hero speed (the enemy speed's
  table with it), the view following the hero, the arrows taken away, the flags while moving,
  the walk's end (the standing group), `view_shown` 0x6989C0 around a move, the step sounds'
  calls (for T5), a boat sailing (`hero_boat` 0x4CE900); VCMI compared. Recorded: land walks in the eight
  directions at each hero speed, one with the view following, one ended by the movement points
  (the path's rest kept), a sail; each walk's tiles from the original's memory into the sequence.
  T1a2 found the direction keys without Ctrl step the selected hero one tile (design §3.4,
  0x408E97): read with the walk; the clicks join `sequences.toml`'s inputs (the runbook).
- **Acceptance:** §3's walk with its addresses mapped; the sequences recorded twice with the same
  timelines; the Definition of Done.
- **Depends on:** T1a1 (the recording), T1a2 (the hero speed option; both done). **Status:**
  ready.

### E24 M2 T2 — the map over time, headless

- **Scope:** time in the library (beside `live::Moment`, pure, time an input): from a situation
  and a sequence's inputs, the redraws' times, the objects' step, both palette steps and the
  view's corner as the scroll moves it, as the recorded timeline's lines (design §3.4, the cursor
  lines too); the edge cursors (`h3-view`'s `cursor` copied, *provisional*, an origin comment;
  the edge zones as MCL-Q6's answer, the original's behind `--bugs original`); `h3-map-view sequences
  [<id>]` (exit 1 on a difference) and `--frames <dir>`, the harness's frame check against the
  recorded screens, `drawn` over the sequences; unit tests replaying the timelines without the
  original files.
- **Acceptance:** T1a's sequences as recorded, every line and every frame exact outside the
  panels (or each difference an `MCL-Q<n>`); `clients/scripts/check.sh`; the Definition of Done.
- **Depends on:** T1a1, T1a2 (both done). **Status:** ready.

### E24 M2 T3 — the walk drawn

- **Scope:** the walk in T2's timeline (a hero walks given tiles: built for real, M4 and M5 give
  it the engine's tiles): the moving hero's group, frame and pixel offset per frame at the
  speed's times, the view following, the arrows taken away, the walk's end; the moving hero's
  pieces in passes 5, 7 and 9 and the boat sailing; `sequences` and the frame check for walks.
- **Acceptance:** T1b's sequences as recorded, every line and every frame (or each difference an
  `MCL-Q<n>`); the Definition of Done.
- **Depends on:** T1b, T2. **Status:** ready after them.

### E24 M2 T4 — the window

- **Scope:** `h3-map-view window <situation | sequence>` through `h3-sdl` (as `h3-view window`,
  E17 design §8.1): the map at the original's frame times at an integer scale; the scroll by the
  keys and the mouse at the edges with the edge cursors (hardware or drawn); a sequence's walks
  replayed; pause, a step, restart, fullscreen, vsync; `--measure`; `--check` reads every frame
  back against the headless one (Xvfb); `h3-view`'s `clock` copied (*provisional*, an origin
  comment); the Windows cross-build, tried on the user's machine.
- **Acceptance:** a situation and a walk sequence play in the window; `--check` exact; the
  Definition of Done.
- **Depends on:** T2, T3. **Status:** ready after them.

### E24 M2 T5 — the map's sound

- **Scope:** readings (`E24-design.md` §3): the adventure music (which file, by what terrain,
  when it changes, its fade), the objects' ambient loops (which objects and sounds, the volume
  by distance, how many at once), the hero's step sounds (T1b's calls; by terrain); the map's
  asset tables' sounds and the gate's check of them; built in the window (`--mute`, keys for the
  music and the effects), on E17's pieces (`h3_assets::snd`, `wav`, `nanomp3`, `h3-sdl`); the
  cues recorded in the original if the harness can tell them (E17 M2 T5b's way).
- **Acceptance:** the sounds as read, the gate's check; the Definition of Done.
- **Depends on:** T4 (the window), T1b (the step sounds' calls). **Status:** ready after them.

### E24 M2 T6 — M2's closure

- **Scope:** M2's exit (the epic) checked on `master`; fresh reviewer agents, one per area, in
  parallel; each finding checked against the code and the executable, fixed or queued; R2-8's
  one-line `cross-lane` pull request if no task took it (the inbox); the review list to the
  user; M2's section moved to `E24-archive.md`; M3 T0 queued (the common crate proposed to the
  client lane first: the epic's M2 decision).
- **Acceptance:** the review list accepted by the user; the Definition of Done.
- **Depends on:** T1a1 to T5. **Status:** ready after them.

## Inbox

Other lanes add entries here. The map client lane turns each into a task, or keeps one line here
while the work waits.

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
